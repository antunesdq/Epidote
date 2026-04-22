use eframe::egui::{
    self, pos2, vec2, Color32, Margin, Rect, Sense, Stroke, TextEdit, Ui, Vec2,
};

use crate::{fonts, icons, theme};

pub fn show(ui: &mut Ui, search: &mut String, menu_collapsed: &mut bool) {
    let bar_rect = ui.max_rect();
    let center_y = bar_rect.center().y;

    // When the sidebar is hidden, show a tiny chevron on the left edge of the
    // top bar to bring it back.
    if *menu_collapsed {
        let btn_d = 28.0;
        let expand_rect = Rect::from_center_size(
            pos2(bar_rect.left() + btn_d / 2.0, center_y),
            Vec2::splat(btn_d),
        );
        if expand_toggle(ui, expand_rect, icons::CHEVRON_RIGHT, "expand_menu").clicked() {
            *menu_collapsed = false;
        }
    }

    // Centered search bar (max-w-md ≈ 448 in Tailwind).
    let search_h = 36.0;
    let search_w = 448.0_f32.min((bar_rect.width() - 320.0).max(240.0));
    let search_rect = Rect::from_center_size(
        pos2(bar_rect.center().x, center_y),
        vec2(search_w, search_h),
    );

    ui.scope(|ui| {
        let v = &mut ui.style_mut().visuals;
        v.widgets.inactive.bg_fill = theme::SURFACE_HIGH;
        v.widgets.inactive.weak_bg_fill = theme::SURFACE_HIGH;
        v.widgets.inactive.bg_stroke = Stroke::NONE;
        v.widgets.inactive.fg_stroke.color = theme::TEXT;
        v.widgets.hovered.bg_fill = theme::SURFACE_HIGH;
        v.widgets.hovered.weak_bg_fill = theme::SURFACE_HIGH;
        v.widgets.hovered.bg_stroke = Stroke::NONE;
        v.widgets.hovered.fg_stroke.color = theme::TEXT;
        v.widgets.active.bg_fill = theme::SURFACE_HIGHEST;
        v.widgets.active.weak_bg_fill = theme::SURFACE_HIGHEST;
        v.widgets.active.bg_stroke = Stroke::new(1.0, theme::PRIMARY_SOFT);
        v.widgets.active.fg_stroke.color = theme::TEXT;
        v.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 255, 136, 80);

        ui.put(
            search_rect,
            TextEdit::singleline(search)
                .hint_text("Search tasks, meetings, architecture...")
                .margin(Margin {
                    left: 34,
                    right: 12,
                    top: 8,
                    bottom: 8,
                }),
        );
    });

    ui.painter().text(
        pos2(search_rect.left() + 16.0, center_y),
        egui::Align2::CENTER_CENTER,
        icons::SEARCH,
        fonts::icon(18.0),
        theme::DIM_TEXT,
    );

    // Right-aligned cluster: notifications, help, avatar.
    let avatar_d = 32.0;
    let btn_d = 36.0;
    let gap = 8.0;

    let avatar_rect = Rect::from_center_size(
        pos2(bar_rect.right() - avatar_d / 2.0, center_y),
        Vec2::splat(avatar_d),
    );
    let painter = ui.painter();
    painter.circle_filled(avatar_rect.center(), avatar_d / 2.0, theme::SURFACE_HIGHEST);
    painter.circle_stroke(
        avatar_rect.center(),
        avatar_d / 2.0,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let help_rect = Rect::from_center_size(
        pos2(avatar_rect.left() - gap - btn_d / 2.0, center_y),
        Vec2::splat(btn_d),
    );
    let notif_rect = Rect::from_center_size(
        pos2(help_rect.left() - gap - btn_d / 2.0, center_y),
        Vec2::splat(btn_d),
    );

    icon_button(ui, notif_rect, "notif", icons::NOTIFICATIONS);
    icon_button(ui, help_rect, "help", icons::HELP);
}

fn icon_button(ui: &mut Ui, rect: Rect, id: &'static str, glyph: &'static str) -> egui::Response {
    let response = ui.interact(rect, egui::Id::new(("top_bar_btn", id)), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 4.0, theme::SURFACE_HIGHEST);
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        fonts::icon(20.0),
        theme::TEXT,
    );
    response
}

fn expand_toggle(
    ui: &mut Ui,
    rect: Rect,
    glyph: &'static str,
    id: &'static str,
) -> egui::Response {
    let response = ui.interact(rect, egui::Id::new(("top_bar_btn", id)), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, color) = if response.hovered() {
        (theme::SURFACE_CONTAINER, theme::PRIMARY)
    } else {
        (theme::SURFACE_HIGH, theme::TEXT)
    };
    painter.rect_filled(rect, 4.0, bg);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        fonts::icon(18.0),
        color,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
