use std::path::PathBuf;

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
};

use crate::{fonts, icons, nav::Open, theme};

const LIST_W: f32 = 320.0;
const ARTIFACTS_W: f32 = 384.0;
const GAP: f32 = 1.0; // panels share a hairline divider instead of a gap
/// Hide the artifacts aside when the outer pane is below this width.
const ARTIFACTS_BREAKPOINT: f32 = 1100.0;
/// Hide the list aside when we're even narrower.
const LIST_BREAKPOINT: f32 = 720.0;

// ----- public state -----

pub struct State {
    meetings: Vec<Meeting>,
    selected: usize,
    search: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            meetings: sample_meetings(),
            selected: 0,
            search: String::new(),
        }
    }
}

// ----- model -----

struct Meeting {
    date_short: &'static str,
    date_long: &'static str,
    title: &'static str,
    preview: &'static str,
    duration: &'static str,
    participant_count: u32,
    live: bool,
    pinned: bool,
    participants: Vec<Participant>,
    transcript: Vec<TranscriptEntry>,
    summary: &'static str,
    actions: Vec<ActionItem>,
    docs: Vec<GeneratedDoc>,
}

struct Participant {
    initials: &'static str,
    name: &'static str,
    accent: Color32,
}

struct TranscriptEntry {
    speaker: &'static str,
    initials: &'static str,
    timestamp: &'static str,
    text: &'static str,
    highlight: Option<Highlight>,
    accent: Color32,
}

struct Highlight {
    quote: &'static str,
    label: &'static str,
}

struct ActionItem {
    title: &'static str,
    owner: &'static str,
    due: &'static str,
    due_soon: bool,
    task_id: Option<&'static str>,
}

struct GeneratedDoc {
    name: &'static str,
    glyph: &'static str,
    vault_path: Option<PathBuf>,
}

const ACCENT_GREEN: Color32 = theme::PRIMARY;
const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xce, 0xbd, 0xff);
const ACCENT_AMBER: Color32 = Color32::from_rgb(0xff, 0xd5, 0x8a);
const ACCENT_GREY: Color32 = theme::OUTLINE;

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let total = ui.available_size_before_wrap();
    let origin = ui.cursor().min;

    ui.allocate_rect(
        Rect::from_min_size(origin, vec2(total.x, total.y)),
        Sense::hover(),
    );

    let mut nav: Option<Open> = None;
    let show_artifacts = total.x >= ARTIFACTS_BREAKPOINT;
    let show_list = total.x >= LIST_BREAKPOINT;

    let artifacts_w = if show_artifacts { ARTIFACTS_W } else { 0.0 };
    let list_w = if show_list { LIST_W } else { 0.0 };
    let dividers = [show_list, show_artifacts].iter().filter(|v| **v).count() as f32;
    let details_w = (total.x - list_w - artifacts_w - dividers * GAP).max(200.0);

    let mut cursor_x = origin.x;

    if show_list {
        let list_rect = Rect::from_min_size(pos2(cursor_x, origin.y), vec2(list_w, total.y));
        let mut list_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(list_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        show_list_pane(&mut list_ui, state);
        cursor_x += list_w + GAP;
    }

    {
        let details_rect =
            Rect::from_min_size(pos2(cursor_x, origin.y), vec2(details_w, total.y));
        let mut details_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(details_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        show_details_pane(&mut details_ui, state);
        cursor_x += details_w + GAP;
    }

    if show_artifacts {
        if let Some(meeting) = state.meetings.get(state.selected) {
            let rect = Rect::from_min_size(pos2(cursor_x, origin.y), vec2(artifacts_w, total.y));
            let mut aside_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::top_down(Align::Min)),
            );
            if let Some(o) = show_artifacts_pane(&mut aside_ui, meeting) {
                nav = Some(o);
            }
        }
    }

    nav
}

// ----- list pane -----

fn show_list_pane(ui: &mut Ui, state: &mut State) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 0.0, theme::SURFACE_CONTAINER_LOW);
    // Hairline right edge
    ui.painter().line_segment(
        [outer.right_top(), outer.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let inner = outer.shrink(18.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    search_box(&mut child, &mut state.search);
    child.add_space(18.0);
    child.label(
        RichText::new("RECENT MEETINGS")
            .font(fonts::display(10.5))
            .strong()
            .color(theme::DIM_TEXT),
    );
    child.add_space(10.0);

    let query = state.search.trim().to_lowercase();
    let filtered: Vec<usize> = state
        .meetings
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            if query.is_empty() {
                return true;
            }
            m.title.to_lowercase().contains(&query)
                || m.preview.to_lowercase().contains(&query)
                || m.date_long.to_lowercase().contains(&query)
        })
        .map(|(i, _)| i)
        .collect();

    egui::ScrollArea::vertical()
        .id_salt("meetings_list_scroll")
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            for i in filtered {
                let m = &state.meetings[i];
                if list_card(ui, m, i == state.selected) {
                    state.selected = i;
                }
                ui.add_space(4.0);
            }
        });
}

fn search_box(ui: &mut Ui, search: &mut String) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        2.0,
        theme::SURFACE_HIGH,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::SEARCH,
        fonts::icon(13.0),
        theme::DIM_TEXT,
    );

    let input_rect = Rect::from_min_size(
        pos2(rect.left() + 30.0, rect.top() + 6.0),
        vec2(rect.width() - 40.0, rect.height() - 12.0),
    );
    let mut input_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(input_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    input_ui.add(
        TextEdit::singleline(search)
            .hint_text("Search archive…")
            .frame(false)
            .desired_width(f32::INFINITY)
            .font(FontId::proportional(12.0)),
    );
}

fn list_card(ui: &mut Ui, m: &Meeting, active: bool) -> bool {
    let height = 78.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let painter = ui.painter_at(rect);

    let (bg, stroke) = if active {
        (
            with_alpha(theme::PRIMARY, 14),
            Stroke::new(1.0, with_alpha(theme::PRIMARY, 110)),
        )
    } else if response.hovered() {
        (theme::SURFACE_CONTAINER, Stroke::NONE)
    } else {
        (Color32::TRANSPARENT, Stroke::NONE)
    };
    painter.rect(rect, 3.0, bg, stroke, StrokeKind::Inside);

    // Date + pin
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        m.date_short,
        FontId::monospace(9.5),
        if active { theme::PRIMARY } else { theme::DIM_TEXT },
    );
    if m.pinned {
        painter.text(
            pos2(rect.right() - 10.0, rect.top() + 10.0),
            Align2::RIGHT_TOP,
            icons::PUSH_PIN,
            fonts::icon(12.0),
            if active { theme::PRIMARY } else { theme::DIM_TEXT },
        );
    }

    // Title
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 26.0),
        Align2::LEFT_TOP,
        m.title,
        fonts::display(13.0),
        theme::TEXT,
    );

    // Preview (clipped to a single truncated line).
    let preview = truncate(m.preview, 48);
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 48.0),
        Align2::LEFT_TOP,
        preview,
        FontId::proportional(11.0),
        theme::DIM_TEXT,
    );

    response.clicked()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

// ----- details pane -----

fn show_details_pane(ui: &mut Ui, state: &State) {
    let outer = ui.max_rect();
    ui.painter().rect_filled(outer, 0.0, theme::BACKGROUND);

    let Some(meeting) = state.meetings.get(state.selected) else {
        return;
    };

    // Header (non-scrolling)
    let header_h = 88.0;
    let header_rect = Rect::from_min_size(outer.min, vec2(outer.width(), header_h));
    let mut header_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(header_rect.shrink2(vec2(28.0, 18.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    show_header(&mut header_ui, meeting);
    ui.painter().line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Transcript
    let body_rect = Rect::from_min_size(
        pos2(outer.left(), outer.top() + header_h),
        vec2(outer.width(), outer.height() - header_h),
    );
    let inner_margin = 28.0;
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body_rect.shrink2(vec2(inner_margin, 18.0)))
            .layout(Layout::top_down(Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("meeting_transcript_scroll")
        .auto_shrink([false, false])
        .show(&mut body_ui, |ui| {
            for (i, entry) in meeting.transcript.iter().enumerate() {
                transcript_entry(ui, entry);
                if i + 1 < meeting.transcript.len() {
                    ui.add_space(20.0);
                }
            }

            ui.add_space(28.0);
            separator_label(ui, "MEETING CONTINUED");
            ui.add_space(8.0);
        });
}

fn show_header(ui: &mut Ui, m: &Meeting) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(m.title)
                    .font(fonts::display(22.0))
                    .strong()
                    .color(theme::TEXT),
            );
            if m.live {
                ui.add_space(8.0);
                live_pill(ui);
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            meta_chip(ui, icons::CALENDAR_MONTH, m.date_long);
            ui.add_space(16.0);
            meta_chip(ui, icons::SCHEDULE, m.duration);
            ui.add_space(16.0);
            meta_chip(
                ui,
                icons::PERSON,
                &format!("{} Participants", m.participant_count),
            );
        });
    });
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        icon_button(ui, icons::MORE_VERT);
        ui.add_space(4.0);
        icon_button(ui, icons::SHARE);
    });
}

fn live_pill(ui: &mut Ui) {
    let font = FontId::monospace(9.5);
    let label = "LIVE ARCHIVE";
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), theme::PRIMARY);
    let w = galley.size().x + 16.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        2.0,
        with_alpha(theme::PRIMARY, 22),
        Stroke::new(1.0, with_alpha(theme::PRIMARY, 80)),
        StrokeKind::Inside,
    );
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, theme::PRIMARY);
}

fn meta_chip(ui: &mut Ui, glyph: &str, text: &str) {
    let font = FontId::monospace(10.5);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), theme::DIM_TEXT);
    let w = galley.size().x + 20.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(12.0),
        theme::DIM_TEXT,
    );
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font,
        theme::DIM_TEXT,
    );
}

fn icon_button(ui: &mut Ui, glyph: &str) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        fonts::icon(16.0),
        if response.hovered() {
            theme::TEXT
        } else {
            theme::DIM_TEXT
        },
    );
}

// ----- transcript -----

fn transcript_entry(ui: &mut Ui, e: &TranscriptEntry) {
    ui.horizontal_top(|ui| {
        // Avatar
        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
        let ap = ui.painter_at(avatar_rect);
        ap.circle_filled(avatar_rect.center(), 20.0, with_alpha(e.accent, 40));
        ap.circle_stroke(
            avatar_rect.center(),
            20.0,
            Stroke::new(1.0, with_alpha(e.accent, 160)),
        );
        ap.text(
            avatar_rect.center(),
            Align2::CENTER_CENTER,
            e.initials,
            FontId::proportional(13.0),
            e.accent,
        );

        ui.add_space(14.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(e.speaker)
                        .font(fonts::display(13.0))
                        .strong()
                        .color(theme::TEXT),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(e.timestamp)
                        .font(FontId::monospace(10.0))
                        .color(theme::DIM_TEXT),
                );
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(e.text)
                    .size(13.5)
                    .color(theme::TEXT),
            );
            if let Some(h) = &e.highlight {
                ui.add_space(10.0);
                highlight_callout(ui, h);
            }
        });
    });
}

fn highlight_callout(ui: &mut Ui, h: &Highlight) {
    egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::NONE)
        .inner_margin(Margin {
            left: 14,
            right: 14,
            top: 10,
            bottom: 10,
        })
        .show(ui, |ui| {
            let cursor = ui.cursor().min;
            let height = 62.0;
            // Left accent bar
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(cursor.x - 14.0, cursor.y - 10.0), vec2(2.0, height)),
                0.0,
                theme::PRIMARY,
            );
            ui.label(
                RichText::new(format!("\u{201C}{}\u{201D}", h.quote))
                    .italics()
                    .size(12.5)
                    .color(theme::DIM_TEXT),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("AI HIGHLIGHT · {}", h.label))
                    .font(FontId::monospace(9.5))
                    .strong()
                    .color(theme::PRIMARY),
            );
        });
}

fn separator_label(ui: &mut Ui, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 14.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let mid_y = rect.center().y;
    let font = FontId::monospace(9.5);
    let galley = painter.layout_no_wrap(label.to_string(), font.clone(), theme::DIM_TEXT);
    let text_w = galley.size().x + 20.0;
    let line_w = (rect.width() - text_w) / 2.0;
    painter.line_segment(
        [pos2(rect.left(), mid_y), pos2(rect.left() + line_w, mid_y)],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );
    painter.line_segment(
        [pos2(rect.right() - line_w, mid_y), pos2(rect.right(), mid_y)],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );
    painter.text(
        pos2(rect.center().x, mid_y),
        Align2::CENTER_CENTER,
        label,
        font,
        theme::DIM_TEXT,
    );
}

// ----- artifacts pane -----

fn show_artifacts_pane(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    let outer = ui.max_rect();
    ui.painter().rect_filled(outer, 0.0, theme::SURFACE_CONTAINER);
    ui.painter().line_segment(
        [outer.left_top(), outer.left_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let inner = outer.shrink(20.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    let mut nav: Option<Open> = None;

    egui::ScrollArea::vertical()
        .id_salt("artifacts_scroll")
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            section_header(ui, icons::AUTO_AWESOME, "AI SUMMARY");
            ui.add_space(8.0);
            egui::Frame::default()
                .fill(theme::SURFACE_HIGH)
                .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.label(RichText::new(m.summary).size(12.0).color(theme::TEXT));
                });

            ui.add_space(22.0);
            section_header(ui, icons::TASK_ALT, "ACTION ITEMS");
            ui.add_space(8.0);
            for action in &m.actions {
                if let Some(o) = action_row(ui, action) {
                    nav = Some(o);
                }
                ui.add_space(6.0);
            }

            ui.add_space(22.0);
            section_header(ui, icons::ARTICLE, "GENERATED DOCS");
            ui.add_space(8.0);
            for doc in &m.docs {
                if let Some(o) = doc_row(ui, doc) {
                    nav = Some(o);
                }
                ui.add_space(6.0);
            }

            ui.add_space(22.0);
            section_header(ui, icons::GROUPS, "PARTICIPANTS");
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for p in &m.participants {
                    participant_chip(ui, p);
                }
            });
        });

    nav
}

fn section_header(ui: &mut Ui, glyph: &str, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            fonts::icon(16.0),
            theme::PRIMARY,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(label)
                .font(fonts::display(11.0))
                .strong()
                .color(theme::TEXT),
        );
    });
}

fn action_row(ui: &mut Ui, a: &ActionItem) -> Option<Open> {
    let h = 52.0;
    let has_target = a.task_id.is_some();
    let sense = if has_target {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), sense);
    let painter = ui.painter_at(rect);
    let hovered = response.hovered() && has_target;
    painter.rect(
        rect,
        2.0,
        if hovered {
            theme::SURFACE_HIGHEST
        } else {
            theme::SURFACE_HIGH
        },
        Stroke::new(
            1.0,
            if hovered {
                with_alpha(theme::PRIMARY, 120)
            } else {
                theme::OUTLINE_VARIANT
            },
        ),
        StrokeKind::Inside,
    );
    // Checkbox
    let cb = Rect::from_min_size(pos2(rect.left() + 12.0, rect.center().y - 8.0), vec2(16.0, 16.0));
    painter.rect(
        cb,
        2.0,
        theme::SURFACE_CONTAINER_LOW,
        Stroke::new(
            1.0,
            if hovered {
                theme::PRIMARY
            } else {
                theme::OUTLINE_VARIANT
            },
        ),
        StrokeKind::Inside,
    );
    if hovered {
        painter.text(
            cb.center(),
            Align2::CENTER_CENTER,
            icons::CHECK,
            fonts::icon(11.0),
            with_alpha(theme::PRIMARY, 120),
        );
    }
    // Title (leave room for the open-in-new glyph on the right when clickable).
    let title_right = if has_target { 28.0 } else { 12.0 };
    painter.text(
        pos2(rect.left() + 38.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        a.title,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    if has_target {
        painter.text(
            pos2(rect.right() - 12.0, rect.top() + 10.0),
            Align2::RIGHT_TOP,
            icons::OPEN_IN_NEW,
            fonts::icon(12.0),
            if hovered {
                theme::PRIMARY
            } else {
                theme::DIM_TEXT
            },
        );
    }
    let _ = title_right;
    // Meta
    let owner_font = FontId::monospace(9.5);
    painter.text(
        pos2(rect.left() + 38.0, rect.top() + 28.0),
        Align2::LEFT_TOP,
        format!("OWNER · {}", a.owner),
        owner_font.clone(),
        theme::DIM_TEXT,
    );
    let due_color = if a.due_soon {
        theme::PRIMARY
    } else {
        theme::DIM_TEXT
    };
    painter.text(
        pos2(rect.right() - 12.0, rect.top() + 28.0),
        Align2::RIGHT_TOP,
        a.due,
        owner_font,
        due_color,
    );

    if response.clicked() {
        if let Some(id) = a.task_id {
            return Some(Open::Task(id.to_string()));
        }
    }
    None
}

fn doc_row(ui: &mut Ui, d: &GeneratedDoc) -> Option<Open> {
    let h = 36.0;
    let has_target = d.vault_path.is_some();
    let sense = if has_target {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), sense);
    let painter = ui.painter_at(rect);
    let hovered = response.hovered() && has_target;
    let stroke = if hovered {
        Stroke::new(1.0, with_alpha(theme::PRIMARY, 120))
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };
    painter.rect(rect, 2.0, theme::SURFACE_CONTAINER_LOW, stroke, StrokeKind::Inside);
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        d.glyph,
        fonts::icon(14.0),
        if hovered { theme::PRIMARY } else { theme::DIM_TEXT },
    );
    painter.text(
        pos2(rect.left() + 34.0, rect.center().y),
        Align2::LEFT_CENTER,
        d.name,
        FontId::proportional(11.5),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(12.0),
        if hovered { theme::PRIMARY } else { theme::DIM_TEXT },
    );

    if response.clicked() {
        if let Some(path) = &d.vault_path {
            return Some(Open::Vault(path.clone()));
        }
    }
    None
}

fn participant_chip(ui: &mut Ui, p: &Participant) {
    let name_font = FontId::proportional(10.5);
    let galley = ui
        .painter()
        .layout_no_wrap(p.name.to_string(), name_font.clone(), theme::TEXT);
    let w = 26.0 + galley.size().x + 16.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 26.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        13.0,
        theme::SURFACE_HIGH,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let dot_center = pos2(rect.left() + 13.0, rect.center().y);
    painter.circle_filled(dot_center, 10.0, with_alpha(p.accent, 55));
    painter.text(
        dot_center,
        Align2::CENTER_CENTER,
        p.initials,
        FontId::proportional(9.5),
        p.accent,
    );
    painter.text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::LEFT_CENTER,
        p.name,
        name_font,
        theme::TEXT,
    );
}

// ----- helpers -----

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// ----- sample data -----

fn vault_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Documents/Epidote/Vault")
}

fn sample_meetings() -> Vec<Meeting> {
    let root = vault_root();
    let doc = |rel: &str| Some(root.join(rel));
    vec![
        Meeting {
            date_short: "OCT 24, 2026",
            date_long: "Oct 24, 2026",
            title: "System Architecture Sync",
            preview:
                "Discussion on migrating legacy microservices to the new event-driven pipeline.",
            duration: "54 min 12 sec",
            participant_count: 5,
            live: true,
            pinned: true,
            participants: vec![
                Participant {
                    initials: "SJ",
                    name: "Sarah J.",
                    accent: ACCENT_GREEN,
                },
                Participant {
                    initials: "DC",
                    name: "David C.",
                    accent: ACCENT_PURPLE,
                },
                Participant {
                    initials: "ML",
                    name: "Marcus L.",
                    accent: ACCENT_AMBER,
                },
                Participant {
                    initials: "RK",
                    name: "Rachel K.",
                    accent: ACCENT_GREY,
                },
            ],
            transcript: vec![
                TranscriptEntry {
                    speaker: "Sarah J.",
                    initials: "SJ",
                    timestamp: "04:12",
                    text: "Alright everyone, let's dive into the core transition strategy. We've \
                           been looking at the latency issues on the current monolith. David, \
                           you mentioned we should start the migration with the entity-indexing \
                           service. What's the rationale there versus the auth layer?",
                    highlight: None,
                    accent: ACCENT_GREEN,
                },
                TranscriptEntry {
                    speaker: "David C.",
                    initials: "DC",
                    timestamp: "04:45",
                    text: "The auth layer is too intertwined right now. If we touch that, we \
                           risk a total service outage. The indexing service is relatively \
                           isolated and responsible for 40% of our database load. If we can \
                           offload that to an event-driven model using RabbitMQ, we gain \
                           immediate stability.",
                    highlight: Some(Highlight {
                        quote: "indexing service is… responsible for 40% of our database load.",
                        label: "KEY METRIC",
                    }),
                    accent: ACCENT_PURPLE,
                },
                TranscriptEntry {
                    speaker: "Sarah J.",
                    initials: "SJ",
                    timestamp: "06:10",
                    text: "Makes sense. Let's draft the POC for that first. We'll need Marcus \
                           to look at the schema mapping because we'll be changing how metadata \
                           is stored on the graph nodes.",
                    highlight: None,
                    accent: ACCENT_GREEN,
                },
            ],
            summary:
                "The team discussed decoupling the Entity Indexing Service from the monolith to \
                 alleviate database pressure. High priority was placed on isolation and \
                 scalability. Sarah J. tasked the team with a POC focusing on RabbitMQ \
                 implementation.",
            actions: vec![
                ActionItem {
                    title: "Draft Indexing POC Architecture",
                    owner: "Sarah J.",
                    due: "OCT 27",
                    due_soon: true,
                },
                ActionItem {
                    title: "Schema Mapping Review",
                    owner: "Marcus L.",
                    due: "OCT 30",
                    due_soon: false,
                },
            ],
            docs: vec![
                GeneratedDoc {
                    name: "Migration_Manifesto_v1.md",
                    glyph: icons::DESCRIPTION,
                },
                GeneratedDoc {
                    name: "System_Diagram_Draft.svg",
                    glyph: icons::ACCOUNT_TREE,
                },
            ],
        },
        Meeting {
            date_short: "OCT 22, 2026",
            date_long: "Oct 22, 2026",
            title: "Q3 Roadmap Planning",
            preview:
                "Priority alignment for upcoming core product updates and AI integrations.",
            duration: "1 hr 06 min",
            participant_count: 7,
            live: false,
            pinned: false,
            participants: vec![
                Participant {
                    initials: "SJ",
                    name: "Sarah J.",
                    accent: ACCENT_GREEN,
                },
                Participant {
                    initials: "PL",
                    name: "Priya L.",
                    accent: ACCENT_PURPLE,
                },
                Participant {
                    initials: "DC",
                    name: "David C.",
                    accent: ACCENT_PURPLE,
                },
            ],
            transcript: vec![
                TranscriptEntry {
                    speaker: "Priya L.",
                    initials: "PL",
                    timestamp: "02:30",
                    text: "The top-line theme for Q3 is retrieval quality. Everything else gets \
                           sequenced around that.",
                    highlight: None,
                    accent: ACCENT_PURPLE,
                },
                TranscriptEntry {
                    speaker: "Sarah J.",
                    initials: "SJ",
                    timestamp: "03:05",
                    text: "Agreed. That means TSK-092 (chunking) is blocker-adjacent — if we \
                           don't land on a default window, nothing downstream ships.",
                    highlight: Some(Highlight {
                        quote: "TSK-092 is blocker-adjacent — if we don't land on a default window, nothing downstream ships.",
                        label: "DEPENDENCY",
                    }),
                    accent: ACCENT_GREEN,
                },
            ],
            summary:
                "Q3 anchored on retrieval quality. TSK-092 (chunking defaults) is the critical \
                 dependency. Roadmap re-sequenced so the RAG pipeline cutover lands before the \
                 caching rewrite.",
            actions: vec![ActionItem {
                title: "Publish Q3 roadmap one-pager",
                owner: "Priya L.",
                due: "OCT 26",
                due_soon: true,
            }],
            docs: vec![GeneratedDoc {
                name: "Q3_Roadmap_v0.md",
                glyph: icons::DESCRIPTION,
            }],
        },
        Meeting {
            date_short: "OCT 20, 2026",
            date_long: "Oct 20, 2026",
            title: "Security Audit Review",
            preview:
                "Analyzing the quarterly penetration testing results and remediation steps.",
            duration: "42 min 08 sec",
            participant_count: 4,
            live: false,
            pinned: false,
            participants: vec![
                Participant {
                    initials: "RK",
                    name: "Rachel K.",
                    accent: ACCENT_GREY,
                },
                Participant {
                    initials: "SJ",
                    name: "Sarah J.",
                    accent: ACCENT_GREEN,
                },
            ],
            transcript: vec![TranscriptEntry {
                speaker: "Rachel K.",
                initials: "RK",
                timestamp: "01:05",
                text: "Two highs, five mediums. The highs are both on the ingress gateway — \
                       same class of input-validation miss.",
                highlight: None,
                accent: ACCENT_GREY,
            }],
            summary:
                "Quarterly pentest surfaced two high-severity findings on the ingress gateway \
                 (shared root cause) plus five mediums. Remediation plan to land before the \
                 November release.",
            actions: vec![ActionItem {
                title: "Patch ingress input-validation path",
                owner: "Rachel K.",
                due: "OCT 28",
                due_soon: true,
            }],
            docs: vec![GeneratedDoc {
                name: "Pentest_Q3_Findings.md",
                glyph: icons::DESCRIPTION,
            }],
        },
        Meeting {
            date_short: "OCT 18, 2026",
            date_long: "Oct 18, 2026",
            title: "Frontend Performance Sync",
            preview:
                "Optimizing bundle sizes and improving LCP across the dashboard modules.",
            duration: "38 min 50 sec",
            participant_count: 3,
            live: false,
            pinned: false,
            participants: vec![Participant {
                initials: "DC",
                name: "David C.",
                accent: ACCENT_PURPLE,
            }],
            transcript: vec![TranscriptEntry {
                speaker: "David C.",
                initials: "DC",
                timestamp: "00:45",
                text: "LCP is 2.8s on the dashboard — mostly the graph view's deferred \
                       hydration. We can split the bundle and lazy-load the inspector.",
                highlight: None,
                accent: ACCENT_PURPLE,
            }],
            summary:
                "Dashboard LCP traced to graph-view hydration. Plan: code-split the inspector \
                 and lazy-load the force-directed layout worker. Expected 900ms LCP win.",
            actions: vec![],
            docs: vec![],
        },
    ]
}
