use eframe::egui::{
    self, pos2, vec2, Align, Color32, CursorIcon, FontId, Layout, Rect, RichText, Sense, Ui, Vec2,
};

use crate::{fonts, icons, theme};

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Nav {
    #[default]
    Recordings,
    Meetings,
    Tasks,
    Vault,
    Graph,
    Calendar,
    Integrations,
    Settings,
}

impl Nav {
    pub fn label(self) -> &'static str {
        match self {
            Nav::Recordings => "Recordings",
            Nav::Meetings => "Meetings",
            Nav::Tasks => "Tasks",
            Nav::Vault => "Vault",
            Nav::Graph => "Graph",
            Nav::Calendar => "Calendar",
            Nav::Integrations => "Integrations",
            Nav::Settings => "Settings",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Nav::Recordings => icons::MIC,
            Nav::Meetings => icons::GROUPS,
            Nav::Tasks => icons::CHECK_CIRCLE,
            Nav::Vault => icons::LOCK,
            Nav::Graph => icons::ACCOUNT_TREE,
            Nav::Calendar => icons::CALENDAR_TODAY,
            Nav::Integrations => icons::EXTENSION,
            Nav::Settings => icons::SETTINGS,
        }
    }

    /// Label for the bottom-of-rail creation button. `None` hides the button
    /// entirely — only Settings shows at the bottom for those screens.
    fn cta(self) -> Option<&'static str> {
        match self {
            Nav::Recordings => Some("Record New"),
            Nav::Calendar => Some("Add Event"),
            Nav::Tasks => Some("Add Task"),
            Nav::Vault => Some("Add Entry"),
            Nav::Meetings | Nav::Graph | Nav::Integrations | Nav::Settings => None,
        }
    }
}

pub fn show(ui: &mut Ui, selected: &mut Nav, collapsed: &mut bool) {
    show_header(ui, collapsed);
    ui.add_space(16.0);

    for nav in [
        Nav::Recordings,
        Nav::Meetings,
        Nav::Tasks,
        Nav::Vault,
        Nav::Graph,
        Nav::Calendar,
        Nav::Integrations,
    ] {
        nav_item(ui, nav, selected);
        ui.add_space(2.0);
    }

    // Pin Settings + (optional) CTA to the bottom of the rail.
    let cta_label = selected.cta();
    let cta_block = if cta_label.is_some() { 16.0 /* gap */ + 48.0 /* cta */ } else { 0.0 };
    let bottom_block = 40.0 /* settings */ + cta_block;
    let spacer = (ui.available_height() - bottom_block).max(8.0);
    ui.add_space(spacer);

    nav_item(ui, Nav::Settings, selected);
    if let Some(label) = cta_label {
        ui.add_space(16.0);
        cta_button(ui, label);
    }
}

fn show_header(ui: &mut Ui, collapsed: &mut bool) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(avatar_rect, 4.0, theme::SURFACE_HIGHEST);
        painter.text(
            avatar_rect.center(),
            egui::Align2::CENTER_CENTER,
            icons::HUB,
            fonts::icon(24.0),
            theme::PRIMARY,
        );

        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(
                RichText::new("Epidote")
                    .font(fonts::display(20.0))
                    .strong()
                    .color(theme::PRIMARY),
            );
            ui.add_space(-4.0);
            ui.label(
                RichText::new("Technical Intelligence")
                    .size(11.0)
                    .color(theme::DIM_TEXT),
            );
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if toggle_button(ui, icons::CHEVRON_LEFT).clicked() {
                *collapsed = true;
            }
        });
    });
}

fn toggle_button(ui: &mut Ui, glyph: &'static str) -> egui::Response {
    let size = 24.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter_at(rect);
    let color = if response.hovered() { theme::PRIMARY } else { theme::DIM_TEXT };
    if response.hovered() {
        painter.rect_filled(rect, 4.0, theme::SURFACE_CONTAINER);
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        fonts::icon(18.0),
        color,
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn nav_item(ui: &mut Ui, nav: Nav, selected: &mut Nav) {
    let active = *selected == nav;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());
    let painter = ui.painter_at(rect);
    let hovered = response.hovered();

    let bg = if active || hovered {
        theme::SURFACE_CONTAINER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 2.0, bg);

    if active {
        let accent = Rect::from_min_size(rect.min, vec2(2.0, rect.height()));
        painter.rect_filled(accent, 0.0, theme::PRIMARY);
    }

    let text_color = if active || hovered {
        theme::PRIMARY
    } else {
        Color32::from_rgba_unmultiplied(theme::TEXT.r(), theme::TEXT.g(), theme::TEXT.b(), 153)
    };

    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        nav.glyph(),
        fonts::icon(18.0),
        text_color,
    );
    painter.text(
        pos2(rect.left() + 40.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        nav.label(),
        FontId::proportional(13.0),
        text_color,
    );

    if response.clicked() {
        *selected = nav;
    }
}

fn cta_button(ui: &mut Ui, label: &str) {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 48.0), Sense::click());
    let painter = ui.painter_at(rect);
    let fill = if response.hovered() {
        theme::PRIMARY_SOFT
    } else {
        theme::PRIMARY
    };
    painter.rect_filled(rect, 2.0, fill);

    let icon_size = 18.0;
    let label_font = fonts::display(13.5);
    let label_galley =
        painter.layout_no_wrap(label.to_owned(), label_font.clone(), theme::BUTTON_TEXT);
    let spacing = 8.0;
    let total = icon_size + spacing + label_galley.size().x;
    let cursor_x = rect.center().x - total / 2.0;

    painter.text(
        pos2(cursor_x + icon_size / 2.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icons::ADD,
        fonts::icon(icon_size),
        theme::BUTTON_TEXT,
    );
    painter.text(
        pos2(
            cursor_x + icon_size + spacing,
            rect.center().y - label_galley.size().y / 2.0,
        ),
        egui::Align2::LEFT_TOP,
        label,
        label_font,
        theme::BUTTON_TEXT,
    );
}
