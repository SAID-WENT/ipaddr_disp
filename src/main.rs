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
        // クロージャの戻り値は Ok(...) でラップする仕様です
        Box::new(|_cc| Ok(Box::new(IpApp::default()))),
    )
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
    // 0.35.0 では `update` ではなく `ui` メソッドを実装します
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

/// 外部ネットワークに接続されているローカルIPを取得する関数
fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}