//#![windows_subsystem = "windows"] // ★ これを追加するとコンソール画面が出なくなる
use eframe::egui;
use std::net::UdpSocket;
use std::time::Duration; // 時間指定用のモジュールを追加
use eframe::egui_wgpu::wgpu; // eframe 内部の wgpu をそのまま使う

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0])
            .with_always_on_top(),

// ★ DX12 に固定する設定
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                eframe::egui_wgpu::WgpuSetupCreateNew {
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
                    device_descriptor: std::sync::Arc::new(|_adapter| {
                        wgpu::DeviceDescriptor::default()
                    }),
                }
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "サイネージ用 IP表示アプリ",
        options,
        Box::new(|cc| {
            if let Some(wgpu_state) = &cc.wgpu_render_state {
                println!("[LOG] レンダリングバックエンド: wgpu");
                // ★ .adapter.get_info() を使って GPU 情報を取得します
                let adapter_info = wgpu_state.adapter.get_info();
                println!("[LOG] バックエンド種別: {:?}", adapter_info.backend); // ★ ここが Dx12 になるか確認
                println!("[LOG] GPU情報: {:?}", adapter_info);
            } else {
                println!("[LOG] レンダリングバックエンド: glow (OpenGL)");
            }

            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(IpApp::default()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
let mut fonts = egui::FontDefinitions::default();

    // プロジェクトルート配下の assets フォルダからフォントデータを直接埋め込む
    let font_bytes = include_bytes!("../assets/NotoSansJP-Regular.ttf");

    fonts.font_data.insert(
        "noto_sans".to_owned(),
        egui::FontData::from_static(font_bytes).into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "noto_sans".to_owned());

    ctx.set_fonts(fonts);
}

struct IpApp {
    ip_address: String,
}

impl Default for IpApp {
    fn default() -> Self {
        Self {
            ip_address: get_local_ip().unwrap_or_else(|| "取得失敗".to_string()),
        }
    }
}

impl eframe::App for IpApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // ★ 5秒ごとに画面描画（ui関数の実行）を予約する設定
        ui.ctx().request_repaint_after(Duration::from_secs(5));

        // 描画が実行されるたびにIPアドレスを最新化
        self.ip_address = get_local_ip().unwrap_or_else(|| "取得失敗".to_string());

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("自PCのIPアドレス").size(20.0));

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(&self.ip_address)
                    .size(36.0)
                    .strong()
                    .color(egui::Color32::from_rgb(0, 150, 255)),
            );

            ui.add_space(20.0);
            // 手動更新ボタンもそのまま残しておきます
            if ui.button("今すぐ更新").clicked() {
                // ボタンを押した時も即座に再描画を呼び出す
            }
        });
    }
}

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}