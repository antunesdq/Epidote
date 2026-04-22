use eframe::egui::{Direction, Layout, RichText, Ui};

use crate::{fonts, theme};

/// Temporary stand-in for each feature screen: the screen's name rendered in
/// Epidote green, centered in the central panel. Replaced per-screen as
/// functionality lands.
pub fn show(ui: &mut Ui, title: &str) {
    ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
        ui.label(
            RichText::new(title)
                .font(fonts::display(56.0))
                .strong()
                .color(theme::PRIMARY),
        );
    });
}
