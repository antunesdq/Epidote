// Retired top-level page — preserved here for Phase 3, when recording becomes
// the live state of a meeting.
#![allow(dead_code)]

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2,
};

use crate::{fonts, icons, theme};

const ERROR: Color32 = Color32::from_rgb(0xff, 0xb4, 0xab);
const TRANSCRIPT_W: f32 = 360.0;
/// Below this parent width the transcript panel is hidden so the waveform and
/// controls below it still have enough room to breathe.
const TRANSCRIPT_BREAKPOINT: f32 = 880.0;

pub fn show(ui: &mut Ui) {
    let total = ui.available_size_before_wrap();
    let origin = ui.cursor().min;
    let gap = 16.0;

    ui.allocate_rect(
        Rect::from_min_size(origin, vec2(total.x, total.y)),
        Sense::hover(),
    );

    let show_transcript_panel = total.x >= TRANSCRIPT_BREAKPOINT;
    let left_w = if show_transcript_panel {
        total.x - TRANSCRIPT_W - gap
    } else {
        total.x
    };

    let left_rect = Rect::from_min_size(origin, vec2(left_w, total.y));
    let mut left = ui.new_child(
        UiBuilder::new()
            .max_rect(left_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    show_machine(&mut left);

    if show_transcript_panel {
        let right_rect = Rect::from_min_size(
            pos2(origin.x + left_w + gap, origin.y),
            vec2(TRANSCRIPT_W, total.y),
        );
        let mut right = ui.new_child(
            UiBuilder::new()
                .max_rect(right_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        show_transcript(&mut right);
    }
}

// ----- left column: status + waveform + controls + extractions -----

fn show_machine(ui: &mut Ui) {
    show_status(ui);
    ui.add_space(16.0);

    // Waveform block (flexible height).
    let extractions_h = 92.0;
    let wave_h = (ui.available_height() - extractions_h - 16.0).max(240.0);
    let (wave_rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), wave_h), Sense::hover());
    show_waveform(ui, wave_rect);

    ui.add_space(16.0);
    show_extractions(ui);
}

fn show_status(ui: &mut Ui) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 56.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Pulsing dot
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 5.0, ERROR);
                    ui.painter().circle_stroke(
                        dot_rect.center(),
                        7.5,
                        Stroke::new(1.0, with_alpha(ERROR, 70)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("System Architecture Sync")
                            .font(fonts::display(24.0))
                            .strong()
                            .color(theme::TEXT),
                    );
                });
                ui.add_space(-2.0);
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    ui.painter().text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        icons::FOLDER,
                        fonts::icon(13.0),
                        theme::DIM_TEXT,
                    );
                    ui.label(
                        RichText::new("Project Alpha / Engineering")
                            .size(12.0)
                            .color(theme::DIM_TEXT),
                    );
                });
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.vertical(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("01:24:03")
                                .font(fonts::display(28.0))
                                .strong()
                                .color(theme::PRIMARY),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("ELAPSED")
                                .size(10.0)
                                .color(theme::DIM_TEXT),
                        );
                    });
                });
            });
        },
    );
}

fn show_waveform(ui: &mut Ui, rect: Rect) {
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        4.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );

    // Soft green radial tint in the top-left, approximated with two faint circles.
    painter.circle_filled(
        rect.min + vec2(60.0, 60.0),
        140.0,
        with_alpha(theme::PRIMARY, 8),
    );

    // Waveform viewport is the top portion; controls bar sits at the bottom.
    let controls_h = 72.0;
    let wave_rect = Rect::from_min_size(
        rect.min + vec2(24.0, 24.0),
        vec2(rect.width() - 48.0, rect.height() - controls_h - 48.0),
    );

    // Bars with pseudo-random heights, deterministic via a seeded LCG so the
    // pattern is stable across frames.
    let bar_w = 2.5;
    let gap = 2.0;
    let n = ((wave_rect.width() + gap) / (bar_w + gap)).floor() as usize;
    let mut seed: u32 = 0xA5F2_C091;
    let mid = wave_rect.center().y;
    for i in 0..n {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let r = (seed >> 16) as f32 / 65536.0;
        let amp = 0.20 + r * 0.80;
        let h = amp * wave_rect.height() * 0.85;
        let alpha = 90 + ((seed >> 8) & 0x7F) as u8;
        let x = wave_rect.left() + i as f32 * (bar_w + gap);
        let bar = Rect::from_min_size(pos2(x, mid - h / 2.0), vec2(bar_w, h));
        painter.rect_filled(bar, 1.0, with_alpha(theme::PRIMARY, alpha));
    }

    // Playhead at 80% along the waveform.
    let playhead_x = wave_rect.left() + wave_rect.width() * 0.8;
    painter.line_segment(
        [
            pos2(playhead_x, wave_rect.top() - 6.0),
            pos2(playhead_x, wave_rect.bottom() + 6.0),
        ],
        Stroke::new(1.0, ERROR),
    );
    painter.circle_filled(pos2(playhead_x, wave_rect.top() - 6.0), 4.0, ERROR);

    // Controls strip
    let controls_rect = Rect::from_min_size(
        pos2(rect.left(), rect.bottom() - controls_h),
        vec2(rect.width(), controls_h),
    );
    painter.rect_filled(controls_rect, 0.0, theme::SURFACE_CONTAINER_LOW);
    painter.line_segment(
        [controls_rect.left_top(), controls_rect.right_top()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let mut controls_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(controls_rect.shrink2(vec2(16.0, 12.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    show_controls(&mut controls_ui);
}

fn show_controls(ui: &mut Ui) {
    let strip_w = ui.max_rect().width();
    let show_speaker_name = strip_w >= 460.0;
    let show_input_info = strip_w >= 560.0;

    // Left: speaker chip
    ui.horizontal(|ui| {
        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        let ap = ui.painter_at(avatar_rect);
        ap.rect(
            avatar_rect,
            4.0,
            theme::SURFACE_HIGHEST,
            Stroke::new(1.0, with_alpha(theme::PRIMARY, 90)),
            StrokeKind::Inside,
        );
        ap.text(
            avatar_rect.center(),
            Align2::CENTER_CENTER,
            "SJ",
            FontId::proportional(12.0),
            theme::TEXT,
        );
        if show_speaker_name {
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Sarah J.")
                        .font(fonts::display(12.5))
                        .strong()
                        .color(theme::TEXT),
                );
                ui.add_space(-2.0);
                ui.label(
                    RichText::new("Speaking")
                        .size(10.0)
                        .color(theme::PRIMARY),
                );
            });
        }
    });

    // Right: transport (+ input info when there's room).
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if show_input_info {
            ui.label(
                RichText::new("Input: Default Mic")
                    .size(11.0)
                    .color(theme::DIM_TEXT),
            );
            ui.add_space(6.0);
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                icons::SETTINGS_VOICE,
                fonts::icon(14.0),
                theme::PRIMARY,
            );
            ui.add_space(20.0);
        }

        transport_button(ui, icons::STOP, 36.0, true);
        ui.add_space(10.0);
        transport_button(ui, icons::PAUSE, 44.0, false);
        ui.add_space(10.0);
        transport_button(ui, icons::MIC_OFF, 36.0, false);
    });
}

fn transport_button(ui: &mut Ui, glyph: &'static str, size: f32, destructive: bool) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg) = if destructive {
        (with_alpha(ERROR, 70), ERROR)
    } else if response.hovered() {
        (theme::SURFACE_HIGHEST, theme::TEXT)
    } else {
        (theme::SURFACE_HIGH, theme::TEXT)
    };
    painter.circle_filled(rect.center(), size / 2.0, bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        fonts::icon(size * 0.45),
        fg,
    );
}

fn show_extractions(ui: &mut Ui) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 76.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        4.0,
        theme::SURFACE_CONTAINER_LOW,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let inner = rect.shrink(14.0);
    let mut inner_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    inner_ui.label(
        RichText::new("LIVE EXTRACTIONS")
            .font(fonts::display(10.0))
            .strong()
            .color(theme::DIM_TEXT),
    );
    inner_ui.add_space(6.0);
    inner_ui.horizontal_wrapped(|ui| {
        chip(ui, icons::CODE, "GraphQL", theme::TEXT, false);
        chip(ui, icons::PERSON, "David Chen", Color32::from_rgb(0xce, 0xbd, 0xff), false);
        chip(ui, icons::DNS, "AWS-East-1", theme::PRIMARY, false);
        chip(ui, icons::PSYCHOLOGY, "Processing…", theme::DIM_TEXT, true);
    });
}

fn chip(ui: &mut Ui, glyph: &'static str, label: &str, accent: Color32, pending: bool) {
    let font = FontId::proportional(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let width = 14.0 + 6.0 + galley.size().x + 24.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let border_alpha = if pending { 40 } else { 90 };
    painter.rect(
        rect,
        4.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, with_alpha(theme::OUTLINE_VARIANT, 200)),
        StrokeKind::Inside,
    );
    let text_color = if pending { theme::DIM_TEXT } else { theme::TEXT };
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(13.0),
        with_alpha(accent, 220),
    );
    painter.text(
        pos2(rect.left() + 30.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    let _ = border_alpha;
}

// ----- right column: transcript -----

fn show_transcript(ui: &mut Ui) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 4.0, theme::SURFACE_CONTAINER_LOW);

    // Header
    let header_h = 44.0;
    let header_rect = Rect::from_min_size(outer.min, vec2(outer.width(), header_h));
    let hp = ui.painter_at(header_rect);
    hp.rect_filled(header_rect, 0.0, theme::SURFACE_CONTAINER);
    hp.line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );
    hp.text(
        pos2(header_rect.left() + 14.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        icons::NOTES,
        fonts::icon(16.0),
        theme::TEXT,
    );
    hp.text(
        pos2(header_rect.left() + 36.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        "Live Transcript",
        fonts::display(13.0),
        theme::TEXT,
    );
    // Auto-scroll pill on the right
    let pill_w = 80.0;
    let pill_h = 18.0;
    let pill_rect = Rect::from_min_size(
        pos2(
            header_rect.right() - pill_w - 12.0,
            header_rect.center().y - pill_h / 2.0,
        ),
        vec2(pill_w, pill_h),
    );
    hp.rect(
        pill_rect,
        2.0,
        with_alpha(theme::PRIMARY, 28),
        Stroke::NONE,
        StrokeKind::Inside,
    );
    hp.text(
        pill_rect.center(),
        Align2::CENTER_CENTER,
        "AUTO-SCROLL",
        FontId::monospace(9.5),
        theme::PRIMARY,
    );

    // Body
    let body_rect = Rect::from_min_size(
        pos2(outer.left(), outer.top() + header_h),
        vec2(outer.width(), outer.height() - header_h),
    );
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body_rect.shrink(14.0))
            .layout(Layout::top_down(Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("recordings_transcript")
        .auto_shrink([false, false])
        .show(&mut body_ui, |ui| {
            transcript_entry(
                ui,
                "DC",
                "David C.",
                "01:21:15",
                false,
                "So the main issue we're seeing with the current architecture is the latency \
                 when querying nested relationships. The GraphQL resolver is hitting the \
                 database sequentially.",
            );
            ui.add_space(14.0);
            transcript_entry(
                ui,
                "SJ",
                "Sarah J.",
                "01:22:04",
                true,
                "Right. We discussed implementing DataLoader to batch those requests. Did we \
                 get a chance to prototype that on the staging environment in AWS-East-1?",
            );
            ui.add_space(14.0);
            transcript_entry(
                ui,
                "SJ",
                "Sarah J.",
                "01:23:48",
                true,
                "If we push that update by end of day, we can monitor the performance metrics \
                 overnight. I'll create a Jira ticket for the migration▌",
            );
        });
}

fn transcript_entry(
    ui: &mut Ui,
    initials: &str,
    name: &str,
    ts: &str,
    is_me: bool,
    body: &str,
) {
    ui.horizontal(|ui| {
        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
        let ap = ui.painter_at(avatar_rect);
        let border = if is_me {
            Stroke::new(1.5, theme::PRIMARY)
        } else {
            Stroke::new(1.0, theme::OUTLINE_VARIANT)
        };
        ap.rect(
            avatar_rect,
            4.0,
            theme::SURFACE_HIGHEST,
            border,
            StrokeKind::Inside,
        );
        ap.text(
            avatar_rect.center(),
            Align2::CENTER_CENTER,
            initials,
            FontId::proportional(10.0),
            theme::TEXT,
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(name)
                .font(fonts::display(12.0))
                .strong()
                .color(if is_me { theme::PRIMARY } else { theme::TEXT }),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(ts)
                .font(FontId::monospace(9.5))
                .color(theme::DIM_TEXT),
        );
    });
    ui.add_space(2.0);
    let indent = 28.0;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 0.0),
        Layout::left_to_right(Align::Min),
        |ui| {
            ui.add_space(indent);
            ui.add_sized(
                vec2(ui.available_width() - indent, 0.0),
                egui::Label::new(
                    RichText::new(body)
                        .size(12.5)
                        .color(if is_me {
                            theme::TEXT
                        } else {
                            with_alpha(theme::TEXT, 200)
                        }),
                )
                .wrap(),
            );
        },
    );
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}
