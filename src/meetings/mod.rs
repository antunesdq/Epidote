use std::path::PathBuf;

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
};

use crate::{fonts, icons, nav::Open, theme};

const LIST_W: f32 = 320.0;
const LIST_BREAKPOINT: f32 = 720.0;

const SOFT: Color32 = theme::SOFT_TEXT;
const MUTED: Color32 = theme::DIM_TEXT;
const ERROR: Color32 = theme::ERROR;
const PURPLE: Color32 = theme::ACCENT_PURPLE;
const AMBER: Color32 = theme::ACCENT_AMBER;

const ACCENT_GREEN: Color32 = theme::PRIMARY;
const ACCENT_PURPLE: Color32 = PURPLE;
const ACCENT_AMBER: Color32 = AMBER;
const ACCENT_GREY: Color32 = theme::OUTLINE;

// ----- public state -----

pub struct State {
    meetings: Vec<Meeting>,
    selected: usize,
    filter: Filter,
    tab: Tab,
    search: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            meetings: sample_meetings(),
            selected: 0,
            filter: Filter::All,
            tab: Tab::Transcript,
            search: String::new(),
        }
    }
}

impl State {
    /// Select a meeting by title (case-insensitive). Resets the tab to a
    /// sensible default per the meeting's state. No-op if not found.
    pub fn select(&mut self, title: &str) {
        let lower = title.to_lowercase();
        if let Some(i) = self
            .meetings
            .iter()
            .position(|m| m.title.to_lowercase() == lower)
        {
            self.selected = i;
            self.tab = match self.meetings[i].state {
                MState::Live => Tab::Transcript,
                _ => Tab::Brief,
            };
        }
    }
}

// ----- model -----

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Live,
    Processing,
    Archive,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Live => "Live",
            Filter::Processing => "Processing",
            Filter::Archive => "Archive",
        }
    }
    fn matches(self, m: &Meeting) -> bool {
        match self {
            Filter::All => true,
            Filter::Live => matches!(m.state, MState::Live),
            Filter::Processing => matches!(m.state, MState::Processing),
            Filter::Archive => matches!(m.state, MState::Archived),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Transcript,
    Brief,
    Proposals,
    Tasks,
    Docs,
}

#[derive(Clone, Copy, PartialEq)]
enum MState {
    Live,
    Processing,
    Archived,
}

struct Participant {
    initials: &'static str,
    #[allow(dead_code)] // shown in tooltips when participant chip lands.
    name: &'static str,
    accent: Color32,
}

struct TranscriptEntry {
    speaker: &'static str,
    initials: &'static str,
    timestamp: &'static str,
    text: &'static str,
    highlight: Option<&'static str>,
    live: bool,
    accent: Color32,
}

struct ActionItem {
    title: &'static str,
    owner: &'static str,
    owner_accent: Color32,
    due: &'static str,
    due_soon: bool,
    task_id: Option<&'static str>,
}

struct GeneratedDoc {
    name: &'static str,
    folder: &'static str,
    updated: &'static str,
    vault_path: Option<PathBuf>,
}

struct MeetingProposal {
    id: &'static str,
    kind_label: &'static str,
    kind_accent: Color32,
    title: &'static str,
    rationale: &'static str,
    confidence: u8,
    generated: &'static str,
}

struct AgendaItem {
    num: &'static str,
    title: &'static str,
    duration: &'static str,
}

struct Meeting {
    id: &'static str,
    title: &'static str,
    state: MState,
    project: &'static str,
    date: &'static str,
    #[allow(dead_code)] // surfaced in calendar/cmd-k joins later.
    date_long: &'static str,
    duration_min: u32,
    elapsed: Option<&'static str>,
    processing_step: Option<&'static str>,
    processing_pct: u8,
    pinned: bool,
    preview: &'static str,
    participants: Vec<Participant>,
    transcript: Vec<TranscriptEntry>,
    summary: Option<&'static str>,
    actions: Vec<ActionItem>,
    docs: Vec<GeneratedDoc>,
    proposals: Vec<MeetingProposal>,
    agenda: Vec<AgendaItem>,
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let total = ui.available_size_before_wrap();
    let origin = ui.cursor().min;

    ui.allocate_rect(
        Rect::from_min_size(origin, vec2(total.x, total.y)),
        Sense::hover(),
    );

    let show_list = total.x >= LIST_BREAKPOINT;
    let mut nav: Option<Open> = None;
    let mut cursor_x = origin.x;

    if show_list {
        let list_rect = Rect::from_min_size(pos2(cursor_x, origin.y), vec2(LIST_W, total.y));
        let mut list_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(list_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        show_list_pane(&mut list_ui, state);
        cursor_x += LIST_W;
    }

    let detail_w = total.x - (cursor_x - origin.x);
    let detail_rect = Rect::from_min_size(pos2(cursor_x, origin.y), vec2(detail_w, total.y));
    let mut detail_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(detail_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    if let Some(o) = show_detail_pane(&mut detail_ui, state) {
        nav = Some(o);
    }

    nav
}

// ----- list pane -----

fn show_list_pane(ui: &mut Ui, state: &mut State) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 0.0, theme::SURFACE_CONTAINER_LOW);
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
    child.add_space(12.0);

    // Filter chips
    child.horizontal(|ui| {
        for f in [Filter::All, Filter::Live, Filter::Processing, Filter::Archive] {
            if filter_chip(ui, f.label(), state.filter == f) {
                state.filter = f;
            }
            ui.add_space(4.0);
        }
    });
    child.add_space(12.0);

    let query = state.search.trim().to_lowercase();

    egui::ScrollArea::vertical()
        .id_salt("meetings_list_scroll")
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            for (i, m) in state.meetings.iter().enumerate() {
                if !state.filter.matches(m) {
                    continue;
                }
                if !query.is_empty()
                    && !m.title.to_lowercase().contains(&query)
                    && !m.preview.to_lowercase().contains(&query)
                {
                    continue;
                }
                if list_card(ui, m, i == state.selected) {
                    state.selected = i;
                    state.tab = match m.state {
                        MState::Live => Tab::Transcript,
                        _ => Tab::Brief,
                    };
                }
                ui.add_space(6.0);
            }
        });
}

fn search_box(ui: &mut Ui, search: &mut String) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
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
        MUTED,
    );

    let input_rect = Rect::from_min_max(
        pos2(rect.left() + 30.0, rect.top() + 6.0),
        pos2(rect.right() - 10.0, rect.bottom() - 6.0),
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

fn filter_chip(ui: &mut Ui, label: &str, active: bool) -> bool {
    let font = FontId::proportional(11.0);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let w = g.size().x + 18.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 24.0), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg, stroke) = if active {
        (
            with_alpha(theme::PRIMARY, 24),
            theme::PRIMARY,
            Stroke::new(1.0, with_alpha(theme::PRIMARY, 100)),
        )
    } else if response.hovered() {
        (theme::SURFACE_HIGH, theme::TEXT, Stroke::NONE)
    } else {
        (Color32::TRANSPARENT, SOFT, Stroke::new(1.0, theme::OUTLINE_VARIANT))
    };
    painter.rect(rect, 12.0, bg, stroke, StrokeKind::Inside);
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, fg);
    response.clicked()
}

fn list_card(ui: &mut Ui, m: &Meeting, active: bool) -> bool {
    let h = 96.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let painter = ui.painter_at(rect);

    let (bg, stroke) = if active {
        (
            with_alpha(theme::PRIMARY, 14),
            Stroke::new(1.0, with_alpha(theme::PRIMARY, 110)),
        )
    } else if response.hovered() {
        (theme::SURFACE_HIGH, Stroke::NONE)
    } else {
        (Color32::TRANSPARENT, Stroke::NONE)
    };
    painter.rect(rect, 4.0, bg, stroke, StrokeKind::Inside);

    // Top row: date + state pill (right) + pinned
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        m.date,
        FontId::monospace(9.5),
        if active { theme::PRIMARY } else { MUTED },
    );
    let mut right_x = rect.right() - 12.0;
    if m.pinned {
        painter.text(
            pos2(right_x, rect.top() + 10.0),
            Align2::RIGHT_TOP,
            icons::PUSH_PIN,
            fonts::icon(11.0),
            if active { theme::PRIMARY } else { MUTED },
        );
        right_x -= 18.0;
    }
    match m.state {
        MState::Live => state_pill_at(
            &painter,
            pos2(right_x, rect.top() + 12.0),
            "LIVE",
            theme::PRIMARY,
            true,
        ),
        MState::Processing => state_pill_at(
            &painter,
            pos2(right_x, rect.top() + 12.0),
            "PROCESSING",
            AMBER,
            false,
        ),
        MState::Archived => {}
    }

    // Title
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 28.0),
        Align2::LEFT_TOP,
        m.title,
        fonts::display(13.0),
        theme::TEXT,
    );

    // Preview — truncated single line
    let preview = truncate(m.preview, 50);
    painter.text(
        pos2(rect.left() + 12.0, rect.top() + 50.0),
        Align2::LEFT_TOP,
        &preview,
        FontId::proportional(11.0),
        SOFT,
    );

    // Bottom meta: duration · participants · tasks
    let meta = if !m.actions.is_empty() {
        format!(
            "{}m  ·  {} people  ·  {} tasks",
            m.duration_min,
            m.participants.len(),
            m.actions.len()
        )
    } else {
        format!("{}m  ·  {} people", m.duration_min, m.participants.len())
    };
    painter.text(
        pos2(rect.left() + 12.0, rect.bottom() - 12.0),
        Align2::LEFT_BOTTOM,
        meta,
        FontId::monospace(9.5),
        MUTED,
    );

    response.clicked()
}

fn state_pill_at(painter: &egui::Painter, top_right: egui::Pos2, label: &str, color: Color32, dot: bool) {
    let font = FontId::monospace(9.5);
    let g = painter.layout_no_wrap(label.to_string(), font.clone(), color);
    let pad_l = if dot { 16.0 } else { 8.0 };
    let w = g.size().x + pad_l + 8.0;
    let h = 16.0;
    let rect = Rect::from_min_size(pos2(top_right.x - w, top_right.y), vec2(w, h));
    painter.rect(
        rect,
        8.0,
        with_alpha(color, 30),
        Stroke::new(1.0, with_alpha(color, 110)),
        StrokeKind::Inside,
    );
    if dot {
        painter.circle_filled(pos2(rect.left() + 8.0, rect.center().y), 3.0, color);
    }
    painter.text(
        pos2(rect.left() + pad_l, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        color,
    );
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

// ----- detail pane -----

fn show_detail_pane(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let outer = ui.max_rect();
    ui.painter().rect_filled(outer, 0.0, theme::BACKGROUND);

    let Some(meeting) = state.meetings.get(state.selected) else {
        return None;
    };

    let mut nav: Option<Open> = None;

    // Sticky header
    let header_h = 92.0;
    let header_rect = Rect::from_min_size(outer.min, vec2(outer.width(), header_h));
    let mut header_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(header_rect.shrink2(vec2(28.0, 18.0)))
            .layout(Layout::top_down(Align::Min)),
    );
    show_header(&mut header_ui, meeting);
    ui.painter().line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Tab bar
    let tab_h = 38.0;
    let tab_rect = Rect::from_min_size(
        pos2(outer.left(), outer.top() + header_h),
        vec2(outer.width(), tab_h),
    );
    let mut tab_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(tab_rect.shrink2(vec2(28.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    show_tab_bar(&mut tab_ui, &mut state.tab, meeting);
    ui.painter().line_segment(
        [tab_rect.left_bottom(), tab_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Body
    let body_rect = Rect::from_min_size(
        pos2(outer.left(), outer.top() + header_h + tab_h),
        vec2(outer.width(), outer.height() - header_h - tab_h),
    );
    let body_inner_margin = 28.0;
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body_rect.shrink2(vec2(body_inner_margin, 18.0)))
            .layout(Layout::top_down(Align::Min)),
    );

    let scroll_id = format!("meeting_detail_{}_{:?}", meeting.id, state.tab as u8);
    egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink([false, false])
        .show(&mut body_ui, |ui| match state.tab {
            Tab::Transcript => transcript_tab(ui, meeting),
            Tab::Brief => {
                if let Some(o) = brief_tab(ui, meeting) {
                    nav = Some(o);
                }
            }
            Tab::Proposals => {
                if let Some(o) = proposals_tab(ui, meeting) {
                    nav = Some(o);
                }
            }
            Tab::Tasks => {
                if let Some(o) = tasks_tab(ui, meeting) {
                    nav = Some(o);
                }
            }
            Tab::Docs => {
                if let Some(o) = docs_tab(ui, meeting) {
                    nav = Some(o);
                }
            }
        });

    nav
}

fn show_header(ui: &mut Ui, m: &Meeting) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(m.title)
                .font(fonts::display(22.0))
                .strong()
                .color(theme::TEXT),
        );
        ui.add_space(8.0);
        match m.state {
            MState::Live => header_pill(ui, "LIVE", theme::PRIMARY, true),
            MState::Processing => {
                let label = if let Some(step) = m.processing_step {
                    format!("PROCESSING · {step}")
                } else {
                    "PROCESSING".to_string()
                };
                header_pill(ui, &label, AMBER, false);
            }
            MState::Archived => header_pill(ui, "ARCHIVED", MUTED, false),
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        meta_chip(ui, icons::FOLDER, m.project);
        ui.add_space(14.0);
        meta_chip(ui, icons::SCHEDULE, &format!("{} min", m.duration_min));
        ui.add_space(14.0);
        meta_chip(
            ui,
            icons::GROUP,
            &format!("{} participants", m.participants.len()),
        );
        ui.add_space(14.0);
        avatar_stack(ui, &m.participants);
    });
}

fn header_pill(ui: &mut Ui, label: &str, color: Color32, dot: bool) {
    let font = FontId::monospace(9.5);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), color);
    let pad_l = if dot { 16.0 } else { 8.0 };
    let w = g.size().x + pad_l + 8.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        9.0,
        with_alpha(color, 24),
        Stroke::new(1.0, with_alpha(color, 100)),
        StrokeKind::Inside,
    );
    if dot {
        painter.circle_filled(pos2(rect.left() + 8.0, rect.center().y), 3.0, color);
    }
    painter.text(
        pos2(rect.left() + pad_l, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        color,
    );
}

fn meta_chip(ui: &mut Ui, glyph: &str, text: &str) {
    let font = FontId::monospace(10.5);
    let g = ui.painter().layout_no_wrap(text.to_string(), font.clone(), MUTED);
    let w = g.size().x + 22.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(12.0),
        MUTED,
    );
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font,
        MUTED,
    );
}

fn avatar_stack(ui: &mut Ui, participants: &[Participant]) {
    let max = 4usize;
    let shown = participants.len().min(max);
    let extra = participants.len().saturating_sub(shown);
    let total = shown + if extra > 0 { 1 } else { 0 };
    let av_d = 22.0;
    let overlap = 8.0;
    let w = av_d + (total as f32 - 1.0).max(0.0) * (av_d - overlap);
    // Allocate a tiny vertical headroom + use the unclipped painter so the
    // 1px circle stroke isn't shaved off the top.
    let (rect, _) = ui.allocate_exact_size(vec2(w, av_d + 2.0), Sense::hover());
    let painter = ui.painter();
    for (i, p) in participants.iter().take(shown).enumerate() {
        let cx = rect.left() + av_d / 2.0 + i as f32 * (av_d - overlap);
        let center = pos2(cx, rect.center().y);
        painter.circle_filled(center, av_d / 2.0, theme::SURFACE_HIGHEST);
        painter.circle_stroke(
            center,
            av_d / 2.0,
            Stroke::new(1.0, with_alpha(p.accent, 160)),
        );
        painter.text(
            center,
            Align2::CENTER_CENTER,
            p.initials,
            FontId::proportional(9.5),
            p.accent,
        );
    }
    if extra > 0 {
        let cx = rect.left() + av_d / 2.0 + shown as f32 * (av_d - overlap);
        let center = pos2(cx, rect.center().y);
        painter.circle_filled(center, av_d / 2.0, theme::SURFACE_CONTAINER);
        painter.circle_stroke(center, av_d / 2.0, Stroke::new(1.0, theme::OUTLINE_VARIANT));
        painter.text(
            center,
            Align2::CENTER_CENTER,
            format!("+{extra}"),
            FontId::proportional(9.0),
            MUTED,
        );
    }
}

fn show_tab_bar(ui: &mut Ui, current: &mut Tab, m: &Meeting) {
    tab_btn(ui, current, Tab::Transcript, icons::GRAPHIC_EQ, "Transcript", None);
    tab_btn(ui, current, Tab::Brief, icons::AUTO_AWESOME, "Brief", None);
    let p_count = m.proposals.len();
    tab_btn(
        ui,
        current,
        Tab::Proposals,
        icons::BOLT,
        "Proposals",
        if p_count > 0 { Some(p_count) } else { None },
    );
    let t_count = m.actions.len();
    tab_btn(
        ui,
        current,
        Tab::Tasks,
        icons::CHECK_CIRCLE,
        "Tasks",
        if t_count > 0 { Some(t_count) } else { None },
    );
    let d_count = m.docs.len();
    tab_btn(
        ui,
        current,
        Tab::Docs,
        icons::DESCRIPTION,
        "Docs",
        if d_count > 0 { Some(d_count) } else { None },
    );
}

fn tab_btn(ui: &mut Ui, current: &mut Tab, tab: Tab, glyph: &str, label: &str, count: Option<usize>) {
    let active = *current == tab;
    let font = FontId::proportional(12.5);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let count_w = if let Some(c) = count {
        let cf = FontId::monospace(9.5);
        let cg = ui.painter().layout_no_wrap(c.to_string(), cf, MUTED);
        cg.size().x + 12.0
    } else {
        0.0
    };
    let w = 22.0 + g.size().x + count_w + 16.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    let painter = ui.painter_at(rect);
    let fg = if active {
        theme::TEXT
    } else if response.hovered() {
        theme::TEXT
    } else {
        SOFT
    };
    painter.text(
        pos2(rect.left() + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(13.0),
        fg,
    );
    painter.text(
        pos2(rect.left() + 26.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        fg,
    );
    if let Some(c) = count {
        let cf = FontId::monospace(9.5);
        let pill_x = rect.right() - 12.0 - count_w + 12.0;
        let pill_rect = Rect::from_center_size(
            pos2(pill_x, rect.center().y),
            vec2(count_w - 4.0, 14.0),
        );
        painter.rect_filled(pill_rect, 7.0, theme::SURFACE_HIGH);
        painter.text(
            pill_rect.center(),
            Align2::CENTER_CENTER,
            c.to_string(),
            cf,
            MUTED,
        );
    }
    if active {
        painter.line_segment(
            [
                pos2(rect.left() + 8.0, rect.bottom() - 1.0),
                pos2(rect.right() - 8.0, rect.bottom() - 1.0),
            ],
            Stroke::new(2.0, theme::PRIMARY),
        );
    }
    if response.clicked() {
        *current = tab;
    }
}

// ----- transcript tab -----

fn transcript_tab(ui: &mut Ui, m: &Meeting) {
    if matches!(m.state, MState::Live) {
        live_strip(ui, m);
        ui.add_space(12.0);
        waveform(ui);
        ui.add_space(20.0);
    } else if matches!(m.state, MState::Processing) {
        processing_strip(ui, m);
        ui.add_space(20.0);
    }

    for (i, e) in m.transcript.iter().enumerate() {
        transcript_entry(ui, e);
        if i + 1 < m.transcript.len() {
            ui.add_space(20.0);
        }
    }
}

fn live_strip(ui: &mut Ui, m: &Meeting) {
    egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(m.elapsed.unwrap_or("00:00:00"))
                            .font(fonts::display(22.0))
                            .strong()
                            .color(theme::PRIMARY),
                    );
                    ui.label(
                        RichText::new("ELAPSED")
                            .font(FontId::monospace(9.5))
                            .color(MUTED),
                    );
                });
                ui.add_space(20.0);
                ui.label(
                    RichText::new("Sarah J. speaking · AWS-East-1")
                        .font(FontId::monospace(11.0))
                        .color(MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    transport_btn(ui, icons::STOP, true);
                    ui.add_space(6.0);
                    transport_btn(ui, icons::PAUSE, false);
                    ui.add_space(6.0);
                    transport_btn(ui, icons::MIC_OFF, false);
                });
            });
        });
}

fn transport_btn(ui: &mut Ui, glyph: &str, danger: bool) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg) = if danger {
        (with_alpha(ERROR, 60), ERROR)
    } else if response.hovered() {
        (theme::SURFACE_HIGHEST, theme::TEXT)
    } else {
        (theme::SURFACE_HIGH, theme::TEXT)
    };
    painter.circle_filled(rect.center(), 15.0, bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        fonts::icon(13.0),
        fg,
    );
}

fn waveform(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), 90.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        4.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );

    let bar_w = 2.5;
    let gap = 2.0;
    let inner = rect.shrink2(vec2(16.0, 14.0));
    let n = ((inner.width() + gap) / (bar_w + gap)).floor() as usize;
    let mid = inner.center().y;
    for i in 0..n {
        let t = i as f32 * 0.4;
        let amp = 0.20 + (t.sin().abs() + (t * 0.32).sin().abs()) * 0.40;
        let h = amp.min(1.0) * inner.height() * 0.85;
        let x = inner.left() + i as f32 * (bar_w + gap);
        let alpha = if i as f32 / n as f32 > 0.78 { 80 } else { 220 };
        painter.rect_filled(
            Rect::from_min_size(pos2(x, mid - h / 2.0), vec2(bar_w, h)),
            1.0,
            with_alpha(theme::PRIMARY, alpha),
        );
    }
    let playhead_x = inner.left() + inner.width() * 0.78;
    painter.line_segment(
        [
            pos2(playhead_x, inner.top() - 2.0),
            pos2(playhead_x, inner.bottom() + 2.0),
        ],
        Stroke::new(1.0, ERROR),
    );
    painter.circle_filled(pos2(playhead_x, inner.top() - 2.0), 3.5, ERROR);
}

fn processing_strip(ui: &mut Ui, m: &Meeting) {
    egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}…",
                            m.processing_step.unwrap_or("Processing")
                        ))
                        .font(fonts::display(15.0))
                        .strong()
                        .color(AMBER),
                    );
                    ui.label(
                        RichText::new("AI POST-PROCESSING")
                            .font(FontId::monospace(9.5))
                            .color(MUTED),
                    );
                });
                ui.add_space(16.0);
                let avail = ui.available_width() - 60.0;
                let (track_rect, _) =
                    ui.allocate_exact_size(vec2(avail, 6.0), Sense::hover());
                let painter = ui.painter_at(track_rect);
                painter.rect_filled(track_rect, 3.0, theme::SURFACE_HIGHEST);
                let fill_w = track_rect.width() * (m.processing_pct as f32 / 100.0);
                painter.rect_filled(
                    Rect::from_min_size(track_rect.min, vec2(fill_w, 6.0)),
                    3.0,
                    AMBER,
                );
                ui.label(
                    RichText::new(format!("{}%", m.processing_pct))
                        .font(FontId::monospace(11.0))
                        .color(SOFT),
                );
            });
        });
}

fn transcript_entry(ui: &mut Ui, e: &TranscriptEntry) {
    ui.horizontal_top(|ui| {
        // Reserve 1px of slack on the cross-axis so the circle stroke isn't
        // clipped by the row's tight bounds, and paint without a child clip.
        let (av_rect, _) = ui.allocate_exact_size(vec2(40.0, 42.0), Sense::hover());
        let ap = ui.painter();
        let center = pos2(av_rect.center().x, av_rect.top() + 20.0);
        ap.circle_filled(center, 20.0, with_alpha(e.accent, 40));
        ap.circle_stroke(center, 20.0, Stroke::new(1.0, with_alpha(e.accent, 160)));
        ap.text(
            center,
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
                        .color(e.accent),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(e.timestamp)
                        .font(FontId::monospace(10.0))
                        .color(MUTED),
                );
                if e.live {
                    ui.add_space(6.0);
                    header_pill(ui, "LIVE", theme::PRIMARY, true);
                }
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(e.text)
                    .size(13.5)
                    .color(theme::TEXT),
            );
            if let Some(h) = e.highlight {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("◆ AI tag · {h}"))
                        .font(FontId::monospace(10.0))
                        .strong()
                        .color(theme::PRIMARY),
                );
            }
        });
    });
}

// ----- brief tab -----

fn brief_tab(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    let mut nav: Option<Open> = None;

    if matches!(m.state, MState::Live) {
        return live_brief_placeholder(ui, m);
    }

    if let Some(s) = m.summary {
        section_eyebrow(ui, "Summary");
        ui.add_space(4.0);
        ui.label(RichText::new(s).size(13.5).color(theme::TEXT));
        ui.add_space(20.0);
    }

    if !m.agenda.is_empty() {
        ui.horizontal(|ui| {
            let (icon_r, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
            ui.painter().text(
                icon_r.center(),
                Align2::CENTER_CENTER,
                icons::EVENT_NOTE,
                fonts::icon(14.0),
                theme::PRIMARY,
            );
            ui.label(
                RichText::new("Proposed follow-up agenda")
                    .font(fonts::display(15.0))
                    .strong()
                    .color(theme::TEXT),
            );
        });
        ui.add_space(8.0);
        for item in &m.agenda {
            agenda_row(ui, item);
            ui.add_space(4.0);
        }
        ui.add_space(20.0);
    }

    if !m.proposals.is_empty() {
        // Strip linking to the proposals tab.
        egui::Frame::default()
            .fill(theme::SURFACE_CONTAINER_LOW)
            .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (icon_r, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                    ui.painter().text(
                        icon_r.center(),
                        Align2::CENTER_CENTER,
                        icons::BOLT,
                        fonts::icon(14.0),
                        AMBER,
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} AI proposal{} from this meeting",
                            m.proposals.len(),
                            if m.proposals.len() == 1 { "" } else { "s" }
                        ))
                        .size(13.0)
                        .color(theme::TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if jump_btn(ui, "Review proposals →") {
                            // Jump to the proposals page with the first proposal.
                            if let Some(first) = m.proposals.first() {
                                nav = Some(Open::Proposal(first.id.to_string()));
                            }
                        }
                    });
                });
            });
    }

    if m.summary.is_none() && m.agenda.is_empty() && m.proposals.is_empty() {
        ui.label(
            RichText::new("No AI brief generated for this meeting.")
                .size(13.0)
                .color(SOFT),
        );
    }

    nav
}

fn live_brief_placeholder(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER_LOW)
        .stroke(Stroke::new(
            1.0,
            with_alpha(theme::OUTLINE_VARIANT, 200),
        ))
        .inner_margin(Margin::same(28))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(icons::HOURGLASS_EMPTY)
                        .font(fonts::icon(28.0))
                        .color(MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("The AI brief is generated after the meeting ends.")
                        .size(13.0)
                        .color(SOFT),
                );
                if !m.proposals.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "{} live proposals already captured — see the Proposals tab.",
                            m.proposals.len()
                        ))
                        .font(FontId::monospace(11.0))
                        .color(MUTED),
                    );
                }
            });
        });
    None
}

fn agenda_row(ui: &mut Ui, item: &AgendaItem) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left() + 4.0, rect.center().y),
        Align2::LEFT_CENTER,
        item.num,
        FontId::monospace(11.0),
        theme::PRIMARY,
    );
    painter.text(
        pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        item.title,
        FontId::proportional(12.5),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 4.0, rect.center().y),
        Align2::RIGHT_CENTER,
        item.duration,
        FontId::monospace(10.5),
        MUTED,
    );
}

// ----- proposals tab -----

fn proposals_tab(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    section_eyebrow(
        ui,
        &format!(
            "AI proposals extracted from this meeting · {}",
            m.proposals.len()
        ),
    );
    ui.add_space(10.0);
    let mut nav: Option<Open> = None;
    if m.proposals.is_empty() {
        ui.label(
            RichText::new("No proposals generated yet.")
                .size(13.0)
                .color(SOFT),
        );
    }
    for (i, p) in m.proposals.iter().enumerate() {
        if proposal_card(ui, p) {
            nav = Some(Open::Proposal(p.id.to_string()));
        }
        if i + 1 < m.proposals.len() {
            ui.add_space(8.0);
        }
    }
    nav
}

fn proposal_card(ui: &mut Ui, p: &MeetingProposal) -> bool {
    let response = egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER_LOW)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(p.kind_label.to_uppercase())
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(p.kind_accent),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}% confidence", p.confidence))
                            .font(FontId::monospace(10.5))
                            .color(SOFT),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(p.generated)
                            .font(FontId::monospace(10.0))
                            .color(MUTED),
                    );
                });
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(p.title)
                    .font(fonts::display(14.5))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(p.rationale).size(12.5).color(SOFT));
        })
        .response;
    let click = ui.interact(response.rect, response.id.with("prop_click"), Sense::click());
    click.clicked()
}

// ----- tasks tab -----

fn tasks_tab(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    section_eyebrow(
        ui,
        &format!(
            "Action items extracted from this meeting · {}",
            m.actions.len()
        ),
    );
    ui.add_space(10.0);
    let mut nav: Option<Open> = None;
    if m.actions.is_empty() {
        ui.label(
            RichText::new("No tasks extracted yet.")
                .size(13.0)
                .color(SOFT),
        );
    }
    for a in &m.actions {
        if drawer_row_action(ui, a) {
            if let Some(id) = a.task_id {
                nav = Some(Open::Task(id.to_string()));
            }
        }
        ui.add_space(6.0);
    }
    nav
}

fn drawer_row_action(ui: &mut Ui, a: &ActionItem) -> bool {
    let h = 40.0;
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
        if hovered { theme::SURFACE_HIGH } else { theme::SURFACE_CONTAINER },
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

    // Work out the right-side slots first so avatar/due/open-icon don't
    // collide when the due string is long ("in 2 days", "tomorrow", "—").
    let right_pad = 14.0;
    let icon_w = 14.0;
    let icon_gap = 10.0;
    let due_font = FontId::monospace(10.0);
    let due_galley =
        painter.layout_no_wrap(a.due.to_string(), due_font.clone(), theme::TEXT);
    let due_w = due_galley.size().x;
    let av_r = 9.0;
    let av_gap = 12.0;

    let icon_right = rect.right() - right_pad;
    let icon_center_x = icon_right - icon_w / 2.0;
    let due_right = icon_center_x - icon_w / 2.0 - icon_gap;
    let due_left = due_right - due_w;
    let av_right = due_left - av_gap;
    let av_center = pos2(av_right - av_r, rect.center().y);

    // Status dot
    painter.circle_filled(pos2(rect.left() + 14.0, rect.center().y), 3.5, theme::PRIMARY);
    // ID
    let id_text = a.task_id.unwrap_or("");
    painter.text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::LEFT_CENTER,
        id_text,
        FontId::monospace(10.5),
        theme::PRIMARY,
    );

    // Title — capped width so it can't run into the avatar cluster.
    let title_left = rect.left() + 88.0;
    let title_max_w = (av_center.x - av_r - 10.0 - title_left).max(60.0);
    let title_font = FontId::proportional(12.0);
    let title_galley = painter.layout(
        a.title.to_string(),
        title_font.clone(),
        theme::TEXT,
        title_max_w,
    );
    painter.galley(
        pos2(title_left, rect.center().y - title_galley.size().y / 2.0),
        title_galley,
        theme::TEXT,
    );

    // Owner avatar
    painter.circle_filled(av_center, av_r, with_alpha(a.owner_accent, 50));
    painter.text(
        av_center,
        Align2::CENTER_CENTER,
        owner_initials(a.owner),
        FontId::proportional(9.0),
        a.owner_accent,
    );
    // Due
    painter.text(
        pos2(due_right, rect.center().y),
        Align2::RIGHT_CENTER,
        &a.due,
        due_font,
        if a.due_soon { theme::PRIMARY } else { MUTED },
    );
    // Open icon
    painter.text(
        pos2(icon_center_x, rect.center().y),
        Align2::CENTER_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(11.0),
        if hovered { theme::PRIMARY } else { MUTED },
    );
    response.clicked()
}

fn owner_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|s| s.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .take(2)
        .collect()
}

// ----- docs tab -----

fn docs_tab(ui: &mut Ui, m: &Meeting) -> Option<Open> {
    section_eyebrow(ui, "Documents generated from this meeting");
    ui.add_space(10.0);
    let mut nav: Option<Open> = None;
    if m.docs.is_empty() {
        ui.label(
            RichText::new("No docs generated.")
                .size(13.0)
                .color(SOFT),
        );
    }
    for d in &m.docs {
        if drawer_row_doc(ui, d) {
            if let Some(p) = &d.vault_path {
                nav = Some(Open::Vault(p.clone()));
            }
        }
        ui.add_space(6.0);
    }
    nav
}

fn drawer_row_doc(ui: &mut Ui, d: &GeneratedDoc) -> bool {
    let h = 40.0;
    let has_target = d.vault_path.is_some();
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
        if hovered { theme::SURFACE_HIGH } else { theme::SURFACE_CONTAINER },
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
    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::DESCRIPTION,
        fonts::icon(13.0),
        if hovered { theme::PRIMARY } else { MUTED },
    );
    painter.text(
        pos2(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        d.name,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 30.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{} · {}", d.folder, d.updated),
        FontId::monospace(10.0),
        MUTED,
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(11.0),
        if hovered { theme::PRIMARY } else { MUTED },
    );
    response.clicked()
}

// ----- shared helpers -----

fn section_eyebrow(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label.to_uppercase())
            .font(FontId::monospace(10.0))
            .strong()
            .color(MUTED),
    );
}

fn jump_btn(ui: &mut Ui, label: &str) -> bool {
    let font = FontId::proportional(12.0);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let w = g.size().x + 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
    let painter = ui.painter_at(rect);
    let bg = if response.hovered() {
        theme::SURFACE_HIGH
    } else {
        theme::SURFACE_CONTAINER
    };
    painter.rect(
        rect,
        3.0,
        bg,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, theme::TEXT);
    response.clicked()
}

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
    let path = |rel: &str| root.join(rel);

    let live_transcript = vec![
        TranscriptEntry {
            speaker: "David C.",
            initials: "DC",
            timestamp: "01:21:15",
            text: "So the main issue we're seeing with the current architecture is the latency \
                   when querying nested relationships. The GraphQL resolver is hitting the \
                   database sequentially.",
            highlight: None,
            live: false,
            accent: ACCENT_PURPLE,
        },
        TranscriptEntry {
            speaker: "Sarah J.",
            initials: "SJ",
            timestamp: "01:22:04",
            text: "Right. We discussed implementing DataLoader to batch those requests. Did we \
                   get a chance to prototype that on the staging environment in AWS-East-1?",
            highlight: Some("AWS-East-1 · staging"),
            live: false,
            accent: ACCENT_GREEN,
        },
        TranscriptEntry {
            speaker: "David C.",
            initials: "DC",
            timestamp: "01:22:38",
            text: "Not yet. I was waiting on David's indexing work to land first so we'd be \
                   measuring the right baseline.",
            highlight: None,
            live: false,
            accent: ACCENT_PURPLE,
        },
        TranscriptEntry {
            speaker: "Sarah J.",
            initials: "SJ",
            timestamp: "01:23:48",
            text: "If we push that update by end of day, we can monitor the performance metrics \
                   overnight. I'll create a Jira ticket for the migration▌",
            highlight: None,
            live: true,
            accent: ACCENT_GREEN,
        },
    ];

    let archived_transcript = |title: &str| {
        vec![
            TranscriptEntry {
                speaker: "Sarah J.",
                initials: "SJ",
                timestamp: "02:30",
                text: "Let me start by framing the problem as we saw it last sprint.",
                highlight: None,
                live: false,
                accent: ACCENT_GREEN,
            },
            TranscriptEntry {
                speaker: "David C.",
                initials: "DC",
                timestamp: "03:45",
                text: Box::leak(
                    format!(
                        "The {} came down to three decisions. First was scope.",
                        title.to_lowercase()
                    )
                    .into_boxed_str(),
                ),
                highlight: None,
                live: false,
                accent: ACCENT_PURPLE,
            },
            TranscriptEntry {
                speaker: "Sarah J.",
                initials: "SJ",
                timestamp: "05:12",
                text: "Agreed. And I think the path forward lands cleanly with what Priya \
                       suggested.",
                highlight: Some("Decision logged"),
                live: false,
                accent: ACCENT_GREEN,
            },
        ]
    };

    vec![
        Meeting {
            id: "mtg-live",
            title: "System Architecture Sync",
            state: MState::Live,
            project: "Project Alpha · Engineering",
            date: "Now",
            date_long: "Apr 22, 2026",
            duration_min: 84,
            elapsed: Some("01:24:03"),
            processing_step: None,
            processing_pct: 0,
            pinned: true,
            preview: "Discussion on migrating legacy microservices to the new event-driven pipeline.",
            participants: vec![
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
                Participant { initials: "DC", name: "David C.", accent: ACCENT_PURPLE },
                Participant { initials: "ML", name: "Marcus L.", accent: ACCENT_AMBER },
                Participant { initials: "RK", name: "Rachel K.", accent: ACCENT_GREY },
                Participant { initials: "MA", name: "You", accent: ACCENT_GREEN },
            ],
            transcript: live_transcript,
            summary: None,
            actions: vec![ActionItem {
                title: "Determine optimal chunking strategy for vector embeddings",
                owner: "You",
                owner_accent: ACCENT_GREEN,
                due: "in 4 days",
                due_soon: true,
                task_id: Some("TSK-092"),
            }],
            docs: vec![],
            proposals: vec![
                MeetingProposal {
                    id: "prop-chunking",
                    kind_label: "Solution",
                    kind_accent: theme::PRIMARY,
                    title: "Use 512-token windows with 64-token overlap as the RAG default",
                    rationale: "Recall@10 peaks at 512/64 on the meetings corpus. Token budget \
                                stays within 18% of the 256-window baseline.",
                    confidence: 82,
                    generated: "3 min ago",
                },
                MeetingProposal {
                    id: "prop-extract",
                    kind_label: "Task",
                    kind_accent: ERROR,
                    title: "Create TSK: Benchmark DataLoader batching on GraphQL resolver",
                    rationale: "Sarah mentioned staging on AWS-East-1 @ 01:23:48 and tied it to \
                                overnight perf monitoring.",
                    confidence: 91,
                    generated: "1 min ago",
                },
            ],
            agenda: vec![],
        },
        Meeting {
            id: "mtg-q3roadmap",
            title: "Q3 Roadmap Planning",
            state: MState::Processing,
            project: "Product",
            date: "6h ago",
            date_long: "Apr 22, 2026",
            duration_min: 66,
            elapsed: None,
            processing_step: Some("Generating agenda"),
            processing_pct: 72,
            pinned: false,
            preview: "Priority alignment for upcoming core product updates and AI integrations.",
            participants: vec![
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
                Participant { initials: "PL", name: "Priya L.", accent: ACCENT_PURPLE },
                Participant { initials: "DC", name: "David C.", accent: ACCENT_PURPLE },
            ],
            transcript: archived_transcript("Q3 Roadmap Planning"),
            summary: Some(
                "Q3 anchored on retrieval quality. TSK-092 (chunking defaults) is the critical \
                 dependency. Roadmap re-sequenced so the RAG pipeline cutover lands before the \
                 caching rewrite.",
            ),
            actions: vec![ActionItem {
                title: "Publish Q3 roadmap one-pager",
                owner: "Priya L.",
                owner_accent: ACCENT_PURPLE,
                due: "in 2 days",
                due_soon: true,
                task_id: Some("TSK-118"),
            }],
            docs: vec![GeneratedDoc {
                name: "Q3 Roadmap v0",
                folder: "Planning",
                updated: "4h ago",
                vault_path: None,
            }],
            proposals: vec![
                MeetingProposal {
                    id: "prop-roadmap",
                    kind_label: "Agenda",
                    kind_accent: PURPLE,
                    title: "Proposed agenda for Q4 Kickoff (next Tuesday)",
                    rationale: "Based on Q3 roadmap carryover, 3 unresolved TSK items, and \
                                Priya's retrieval-quality theme.",
                    confidence: 74,
                    generated: "2h ago",
                },
                MeetingProposal {
                    id: "prop-summary",
                    kind_label: "Summary",
                    kind_accent: SOFT,
                    title: "Q3 Roadmap Planning — summary & decisions",
                    rationale: "Q3 anchored on retrieval quality. Roadmap re-sequenced so the \
                                RAG pipeline cutover lands before the caching rewrite.",
                    confidence: 88,
                    generated: "5h ago",
                },
            ],
            agenda: vec![
                AgendaItem { num: "01", title: "Retrieval quality review (Q3 metrics)", duration: "10 min" },
                AgendaItem { num: "02", title: "TSK-092 chunking default · go/no-go", duration: "8 min" },
                AgendaItem { num: "03", title: "TSK-118 roadmap one-pager walk-through", duration: "12 min" },
                AgendaItem { num: "04", title: "Pentest Q3 remediation status", duration: "7 min" },
                AgendaItem { num: "05", title: "Caching RFC v2 resourcing", duration: "10 min" },
                AgendaItem { num: "06", title: "Open floor", duration: "5 min" },
            ],
        },
        Meeting {
            id: "mtg-q3arch",
            title: "Q3 Data Architecture",
            state: MState::Archived,
            project: "Project Alpha · Engineering",
            date: "2d ago",
            date_long: "Apr 20, 2026",
            duration_min: 54,
            elapsed: None,
            processing_step: None,
            processing_pct: 0,
            pinned: false,
            preview: "Caching layer ownership, TSK-104 scope, and write-through vs per-request.",
            participants: vec![
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
                Participant { initials: "DC", name: "David C.", accent: ACCENT_PURPLE },
                Participant { initials: "ML", name: "Marcus L.", accent: ACCENT_AMBER },
            ],
            transcript: archived_transcript("Q3 Data Architecture"),
            summary: Some(
                "Caching layer ownership transitioned to the platform team. TSK-104 scope \
                 confirmed. Decision pending on write-through vs per-request lookup.",
            ),
            actions: vec![ActionItem {
                title: "Review Redis caching strategy for high-frequency reads",
                owner: "David C.",
                owner_accent: ACCENT_PURPLE,
                due: "in 9 days",
                due_soon: false,
                task_id: Some("TSK-104"),
            }],
            docs: vec![GeneratedDoc {
                name: "Caching RFC v2",
                folder: "Architecture",
                updated: "1w ago",
                vault_path: Some(path("Architecture/Caching RFC v2.md")),
            }],
            proposals: vec![
                MeetingProposal {
                    id: "prop-doc-migration",
                    kind_label: "Document",
                    kind_accent: AMBER,
                    title: "Generated: Migration Manifesto v1.md",
                    rationale: "Full architectural proposal extracted from the meeting \
                                transcript. 4 sections, references 2 vault notes.",
                    confidence: 79,
                    generated: "2d ago",
                },
                MeetingProposal {
                    id: "prop-caching-owner",
                    kind_label: "Task",
                    kind_accent: ERROR,
                    title: "Create TSK: Nominate owner for Caching RFC v2 resourcing",
                    rationale: "Caching rewrite was re-sequenced but no owner was explicitly \
                                assigned in the meeting.",
                    confidence: 68,
                    generated: "2d ago",
                },
            ],
            agenda: vec![],
        },
        Meeting {
            id: "mtg-observability",
            title: "Observability Overhaul",
            state: MState::Archived,
            project: "Platform",
            date: "5d ago",
            date_long: "Apr 17, 2026",
            duration_min: 48,
            elapsed: None,
            processing_step: None,
            processing_pct: 0,
            pinned: false,
            preview: "Migration from syslog forwarders to Datadog agents; security review checkpoint.",
            participants: vec![
                Participant { initials: "RK", name: "Rachel K.", accent: ACCENT_GREY },
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
            ],
            transcript: archived_transcript("Observability Overhaul"),
            summary: Some(
                "Datadog agent rollout is staged behind security review. Once the policy bundle \
                 is signed off, switch the legacy syslog forwarder to the Datadog agent in two \
                 phases.",
            ),
            actions: vec![ActionItem {
                title: "Migrate legacy logging to Datadog cluster",
                owner: "Rachel K.",
                owner_accent: ACCENT_GREY,
                due: "tomorrow",
                due_soon: true,
                task_id: Some("TSK-088"),
            }],
            docs: vec![],
            proposals: vec![],
            agenda: vec![],
        },
        Meeting {
            id: "mtg-pipeline",
            title: "Pipeline Reliability",
            state: MState::Archived,
            project: "Platform",
            date: "1w ago",
            date_long: "Apr 15, 2026",
            duration_min: 38,
            elapsed: None,
            processing_step: None,
            processing_pct: 0,
            pinned: false,
            preview: "Retry/backoff policy, failure-mode taxonomy, worker saturation metrics.",
            participants: vec![
                Participant { initials: "ML", name: "Marcus L.", accent: ACCENT_AMBER },
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
            ],
            transcript: archived_transcript("Pipeline Reliability"),
            summary: Some(
                "Surveyed failure modes on ingest workers. Retry/backoff policy needs explicit \
                 spec — TSK-112 captures this.",
            ),
            actions: vec![ActionItem {
                title: "Spec retry/backoff policy for ingest workers",
                owner: "Marcus L.",
                owner_accent: ACCENT_AMBER,
                due: "in 14 days",
                due_soon: false,
                task_id: Some("TSK-112"),
            }],
            docs: vec![],
            proposals: vec![],
            agenda: vec![],
        },
        Meeting {
            id: "mtg-latency",
            title: "Latency Sweep",
            state: MState::Archived,
            project: "Project Alpha · Engineering",
            date: "2w ago",
            date_long: "Apr 08, 2026",
            duration_min: 52,
            elapsed: None,
            processing_step: None,
            processing_pct: 0,
            pinned: false,
            preview: "Cold-start profiling on inference workers after autoscale-down.",
            participants: vec![
                Participant { initials: "DC", name: "David C.", accent: ACCENT_PURPLE },
                Participant { initials: "SJ", name: "Sarah J.", accent: ACCENT_GREEN },
            ],
            transcript: archived_transcript("Latency Sweep"),
            summary: Some(
                "First-token latency spikes after autoscale-down events. Plan: capture flame \
                 graphs from the next idle cycle.",
            ),
            actions: vec![ActionItem {
                title: "Profile cold-start latency on inference workers",
                owner: "David C.",
                owner_accent: ACCENT_PURPLE,
                due: "—",
                due_soon: false,
                task_id: Some("TSK-096"),
            }],
            docs: vec![],
            proposals: vec![],
            agenda: vec![],
        },
    ]
}
