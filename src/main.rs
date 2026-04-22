mod app;
mod fonts;
mod icons;
mod nav;
mod placeholder;
mod theme;

mod calendar;
mod graph;
mod recordings;
mod integrations;
mod login;
mod meetings;
mod menu;
mod settings;
mod tasks;
mod upper_bar;
mod vault;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Epidote - Technical Intelligence")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Epidote",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            fonts::setup(&cc.egui_ctx);
            Ok(Box::new(app::EpidoteApp::default()))
        }),
    )
}
