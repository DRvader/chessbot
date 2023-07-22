use eframe::egui;

mod app;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ChessBot",
        native_options,
        Box::new(|cc| Box::new(app::ChessBotApp::new(cc))),
    )
    .unwrap();
}
