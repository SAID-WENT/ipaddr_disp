#![windows_subsystem = "windows"] // ★ これを追加するとコンソール画面が出なくなる

use std::fs::OpenOptions;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};
use egui::RichText;
use log::info;
use simplelog::{Config as LogConfig, WriteLogger};

const APP_TITLE: &str = "サイネージ用 IP表示アプリ";
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const FONT_NAME: &str = "noto_sans";
const FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSansJP-Regular.ttf");
const IP_FAILED: &str = "取得失敗";
const IP_ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 150, 255);
const DEFAULT_LOG_FILE: &str = "ipaddr_disp.log";

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

#[derive(Default)]
struct IpApp {
    ip_address: String,
}

impl IpApp {
    fn refresh_ip(&mut self) {
        self.ip_address = get_local_ip().unwrap_or_else(|_| IP_FAILED.to_owned());
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
