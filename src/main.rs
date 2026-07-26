use eframe::egui;
use std::net::UdpSocket;

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
            // ★ 日本語フォント（メイリオ）のセットアップ
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(IpApp::default()))
        }),
    )
}

/// Windowsの日本語フォント（Meiryo）をeguiに読み込ませる関数
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Windows標準のメイリオフォントを読み込む
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\meiryo.ttc") {
        fonts.font_data.insert(
            "meiryo".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );

        // プロポーショナルフォント（通常の文章用）の最優先に設定
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "meiryo".to_owned());

        // 等幅フォント（数字やコード用）の最優先にも設定
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
            if ui.button("更新").clicked() {
                self.ip_address = get_local_ip().unwrap_or_else(|| "取得失敗".to_string());
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