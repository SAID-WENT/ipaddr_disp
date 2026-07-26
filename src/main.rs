use eframe::egui;
use std::net::UdpSocket;
use std::time::Duration; // 時間指定用のモジュールを追加

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0])
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "サイネージ用 IP表示アプリ",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(IpApp::default()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\meiryo.ttc") {
        fonts.font_data.insert(
            "meiryo".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "meiryo".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "meiryo".to_owned());

        ctx.set_fonts(fonts);
    }
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
                ui.ctx().request_repaint();
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