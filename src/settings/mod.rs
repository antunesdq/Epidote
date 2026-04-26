use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, RichText, Sense, Stroke, StrokeKind, Ui,
};

use crate::{fonts, icons, theme};

const SOFT: Color32 = theme::SOFT_TEXT;
const MUTED: Color32 = theme::DIM_TEXT;

struct SettingRow {
    label: &'static str,
    value: &'static str,
    sub: &'static str,
}

const ROWS: &[SettingRow] = &[
    SettingRow {
        label: "Recording input",
        value: "MacBook Pro Microphone",
        sub: "Default system audio source",
    },
    SettingRow {
        label: "AI model",
        value: "epidote-intel-v3",
        sub: "Used for summaries, agendas, and proposals",
    },
    SettingRow {
        label: "Vault location",
        value: "~/Documents/Epidote/Vault",
        sub: "Local-first storage for your knowledge base",
    },
    SettingRow {
        label: "Calendar sync",
        value: "Google · antunes@…",
        sub: "1 connected calendar",
    },
    SettingRow {
        label: "Integrations",
        value: "3 connected",
        sub: "GitHub, Linear, Slack",
    },
    SettingRow {
        label: "Theme",
        value: "System (dark)",
        sub: "Use Tweaks to preview variants",
    },
];

pub fn show(ui: &mut Ui) {
    egui::ScrollArea::vertical()
        .id_salt("settings_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                RichText::new("Settings")
                    .font(fonts::display(28.0))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(
                    "Integrations, audio devices, AI model preferences, and account.",
                )
                .size(13.0)
                .color(SOFT),
            );
            ui.add_space(24.0);

            for row in ROWS {
                setting_row(ui, row);
                ui.add_space(8.0);
            }
        });
}

fn setting_row(ui: &mut Ui, row: &SettingRow) {
    let h = 60.0;
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), h),
        Sense::click(),
    );
    let painter = ui.painter_at(rect);
    let bg = if response.hovered() {
        theme::SURFACE_HIGH
    } else {
        theme::SURFACE_CONTAINER_LOW
    };
    painter.rect(
        rect,
        4.0,
        bg,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );

    // Label + sub on the left.
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        row.label,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 32.0),
        Align2::LEFT_TOP,
        row.sub,
        FontId::proportional(11.0),
        MUTED,
    );

    // Trailing chevron + value
    let chev_pad = 14.0;
    let chev_x = rect.right() - chev_pad;
    painter.text(
        pos2(chev_x, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::CHEVRON_RIGHT,
        fonts::icon(14.0),
        if response.hovered() { theme::TEXT } else { MUTED },
    );

    let value_font = FontId::monospace(11.5);
    painter.text(
        pos2(chev_x - 24.0, rect.center().y),
        Align2::RIGHT_CENTER,
        row.value,
        value_font,
        theme::TEXT,
    );
}
