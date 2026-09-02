#![windows_subsystem = "windows"] // ★ これを追加するとコンソール画面が出なくなる

use std::fs::{self, OpenOptions};
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};
use egui::RichText;
use log::{info, warn};
use serde::Deserialize;
use simplelog::{Config as LogConfig, WriteLogger};

const APP_TITLE: &str = "サイネージ用 IP表示アプリ";
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const FONT_NAME: &str = "noto_sans";
const FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSansJP-Regular.ttf");
const IP_FAILED: &str = "取得失敗";
const IP_ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 150, 255);
const DEFAULT_LOG_FILE: &str = "ipaddr_disp.log";
const CONFIG_FILE: &str = "config.json";

#[derive(Parser)]
#[command(name = "ipaddr_disp", about = "サイネージ用 IP表示アプリ")]
struct Cli {
    /// ログ出力を有効にする。値の省略時は既定ファイル (ipaddr_disp.log) に出力する
    #[arg(long, num_args = 0..=1, default_missing_value = DEFAULT_LOG_FILE)]
    log: Option<String>,
    /// コンソール(標準エラー出力)にもログを出力する
    #[arg(long)]
    console: bool,
}

fn default_output_path() -> PathBuf {
    PathBuf::from("ipaddr.txt")
}

/// アプリ動作を制御する設定。`config.json` から読み込む。
#[derive(Debug, Clone, Deserialize)]
struct AppConfig {
    /// IPアドレスのファイル出力を行うかどうか
    #[serde(default)]
    enable_ip_export: bool,
    /// 出力先テキストファイルのパス
    #[serde(default = "default_output_path")]
    output_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enable_ip_export: false,
            output_path: default_output_path(),
        }
    }
}

/// 起動時に `config.json` の読み込みを試みる。
/// ファイルが存在しない場合は自動生成せず、メモリ上のデフォルト値
/// (`enable_ip_export = false`) を返す。
fn load_config() -> AppConfig {
    let content = match fs::read_to_string(CONFIG_FILE) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            info!("[CONFIG] {CONFIG_FILE} が存在しないためデフォルト設定を使用します");
            return AppConfig::default();
        }
        Err(err) => {
            warn!("[CONFIG] {CONFIG_FILE} を読み込めませんでした ({err})。デフォルト設定を使用します");
            return AppConfig::default();
        }
    };

    match serde_json::from_str::<AppConfig>(&content) {
        Ok(config) => {
            info!(
                "[CONFIG] {CONFIG_FILE} を読み込みました (enable_ip_export={}, output_path={})",
                config.enable_ip_export,
                config.output_path.display()
            );
            config
        }
        Err(err) => {
            warn!("[CONFIG] {CONFIG_FILE} のパースに失敗しました ({err})。デフォルト設定を使用します");
            AppConfig::default()
        }
    }
}

/// `enable_ip_export` が true かつ IP取得成功時のみ、指定パスへ上書き出力する。
fn export_ip(config: &AppConfig, ip: &str) {
    if !config.enable_ip_export {
        return;
    }
    if ip.is_empty() || ip == IP_FAILED {
        info!("[EXPORT] IPアドレスが取得できていないためファイル出力をスキップします");
        return;
    }

    if let Some(parent) = config.output_path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(err) = fs::create_dir_all(parent)
    {
        warn!(
            "[EXPORT] 出力先ディレクトリを作成できませんでした ({}): {err}",
            parent.display()
        );
        return;
    }

    match fs::write(&config.output_path, ip) {
        Ok(()) => info!(
            "[EXPORT] IPアドレス ({ip}) を {} に出力しました",
            config.output_path.display()
        ),
        Err(err) => warn!(
            "[EXPORT] IPアドレスを {} に出力できませんでした: {err}",
            config.output_path.display()
        ),
    }
}

fn main() -> eframe::Result {
    let cli = Cli::parse();
    init_logging(&cli);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0])
            .with_always_on_top(),
        wgpu_options: dx12_wgpu_options(),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| {
            log_render_backend(&cc.wgpu_render_state);
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(IpApp::default()))
        }),
    )
}

fn init_logging(cli: &Cli) {
    if cli.log.is_none() && !cli.console {
        return;
    }

    let level = log::LevelFilter::Info;
    let config = LogConfig::default();
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = Vec::new();

    if cli.console {
        let logger: Box<dyn simplelog::SharedLogger> =
            simplelog::SimpleLogger::new(level, config.clone());
        loggers.push(logger);
    }

    if let Some(path) = cli.log.as_deref() {
        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => {
                let logger: Box<dyn simplelog::SharedLogger> =
                    WriteLogger::new(level, config.clone(), file);
                loggers.push(logger);
            }
            Err(err) => eprintln!("[LOG] ログファイルを開けませんでした ({path}): {err}"),
        }
    }

    if let Err(err) = simplelog::CombinedLogger::init(loggers) {
        eprintln!("[LOG] ロガー初期化失敗: {err}");
    }
}

fn dx12_wgpu_options() -> egui_wgpu::WgpuConfiguration {
    egui_wgpu::WgpuConfiguration {
        wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(egui_wgpu::WgpuSetupCreateNew {
            instance_descriptor: wgpu::InstanceDescriptor {
                backends: wgpu::Backends::DX12,
                flags: wgpu::InstanceFlags::default(),
                backend_options: wgpu::BackendOptions::default(),
                display: None,
                memory_budget_thresholds: Default::default(),
            },
            power_preference: wgpu::PowerPreference::HighPerformance,
            display_handle: None,
            native_adapter_selector: None,
            device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor::default()),
        }),
        ..Default::default()
    }
}

fn log_render_backend(wgpu_state: &Option<egui_wgpu::RenderState>) {
    let Some(wgpu_state) = wgpu_state else {
        info!("[LOG] レンダリングバックエンド: glow (OpenGL)");
        return;
    };

    info!("[LOG] レンダリングバックエンド: wgpu");
    let adapter_info = wgpu_state.adapter.get_info();
    info!("[LOG] バックエンド種別: {:?}", adapter_info.backend);
    info!("[LOG] GPU情報: {:?}", adapter_info);
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert(FONT_NAME.to_owned(), egui::FontData::from_static(FONT_BYTES).into());

    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, FONT_NAME.to_owned());
    }

    ctx.set_fonts(fonts);
}

struct IpApp {
    ip_address: String,
    config: AppConfig,
}

impl Default for IpApp {
    fn default() -> Self {
        let config = load_config();
        let mut app = Self {
            ip_address: String::new(),
            config,
        };
        // アプリ起動時の初回取得＋ファイル出力
        app.refresh_ip();
        app
    }
}

impl IpApp {
    fn refresh_ip(&mut self) {
        self.ip_address = get_local_ip().unwrap_or_else(|_| IP_FAILED.to_owned());
        export_ip(&self.config, &self.ip_address);
    }
}

impl eframe::App for IpApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_ip();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint_after(REFRESH_INTERVAL);

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("自PCのIPアドレス").size(20.0));

            ui.add_space(10.0);
            ui.label(
                RichText::new(&self.ip_address)
                    .size(36.0)
                    .strong()
                    .color(IP_ACCENT),
            );

            ui.add_space(20.0);
            if ui.button("今すぐ更新").clicked() {
                self.refresh_ip();
                ui.ctx().request_repaint();
            }
        });
    }
}

fn get_local_ip() -> std::io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip().to_string())
}
