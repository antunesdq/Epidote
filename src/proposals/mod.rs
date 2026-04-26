use std::path::PathBuf;

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder,
};

use crate::{fonts, icons, nav::Open, theme};

const MUTED: Color32 = theme::DIM_TEXT;
const SOFT: Color32 = theme::SOFT_TEXT;
const ERROR: Color32 = theme::ERROR;
const PURPLE: Color32 = theme::ACCENT_PURPLE;
const AMBER: Color32 = theme::ACCENT_AMBER;

const FOCUS_GRID_BREAKPOINT: f32 = 980.0;

// ----- public state -----

pub struct State {
    proposals: Vec<Proposal>,
    filter: Filter,
    selected: Option<&'static str>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            proposals: sample_proposals(),
            filter: Filter::Pending,
            selected: None,
        }
    }
}

impl State {
    /// Open a proposal in focus mode by id. No-op if the id doesn't exist.
    pub fn select(&mut self, id: &str) {
        if self.proposals.iter().any(|p| p.id == id) {
            // SAFETY: we only ever store ids from sample data, all of which are
            // 'static. The lookup above guaranteed the slice is in our table.
            let static_id = self
                .proposals
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.id)
                .unwrap();
            self.selected = Some(static_id);
        }
    }
}

// ----- model -----

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    Pending,
    Approved,
    All,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Filter::Pending => "Pending",
            Filter::Approved => "Approved",
            Filter::All => "All",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Solution,
    Task,
    Agenda,
    Summary,
    Doc,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Solution => "Solution",
            Kind::Task => "Task",
            Kind::Agenda => "Agenda",
            Kind::Summary => "Summary",
            Kind::Doc => "Document",
        }
    }
    fn accent(self) -> Color32 {
        match self {
            Kind::Solution => theme::PRIMARY,
            Kind::Task => ERROR,
            Kind::Agenda => PURPLE,
            Kind::Summary => SOFT,
            Kind::Doc => AMBER,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PStatus {
    Pending,
    Approved,
}

struct Evidence {
    kind: EvidenceKind,
    text: &'static str,
    meta: &'static str,
    accent: Color32,
    target: Option<Open>,
}

enum EvidenceKind {
    Transcript,
    Vault,
}

struct RelatedTask {
    id: &'static str,
    title: &'static str,
    status: TaskStatus,
}

#[derive(Copy, Clone)]
#[allow(dead_code)] // `Blocked` shows up in sample data variants beyond the seed.
enum TaskStatus {
    Active,
    Triage,
    Blocked,
}

impl TaskStatus {
    fn dot(self) -> Color32 {
        match self {
            TaskStatus::Active => theme::PRIMARY,
            TaskStatus::Triage => theme::OUTLINE,
            TaskStatus::Blocked => ERROR,
        }
    }
}

struct RelatedDoc {
    name: &'static str,
    folder: &'static str,
    vault_path: PathBuf,
}

struct Proposal {
    id: &'static str,
    kind: Kind,
    title: &'static str,
    rationale: &'static str,
    details: Option<&'static str>,
    confidence: u8,
    status: PStatus,
    generated: &'static str,
    meeting: Option<&'static str>,
    evidence: Vec<Evidence>,
    related_tasks: Vec<RelatedTask>,
    related_docs: Vec<RelatedDoc>,
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    if let Some(id) = state.selected {
        if let Some(p) = state.proposals.iter().find(|p| p.id == id) {
            return show_focus(ui, p, &mut state.selected);
        } else {
            state.selected = None;
        }
    }
    show_list(ui, state)
}

// ----- list view -----

fn show_list(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let nav: Option<Open> = None;
    egui::ScrollArea::vertical()
        .id_salt("proposals_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header
            ui.label(
                RichText::new("Proposals")
                    .font(fonts::display(28.0))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(
                    "AI-generated summaries, agendas, tasks, and solutions. Review and accept to turn them into real work.",
                )
                .size(13.0)
                .color(SOFT),
            );
            ui.add_space(20.0);

            // Filter chips
            ui.horizontal(|ui| {
                for f in [Filter::Pending, Filter::Approved, Filter::All] {
                    if filter_chip(ui, f.label(), state.filter == f) {
                        state.filter = f;
                    }
                    ui.add_space(6.0);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let count = state
                        .proposals
                        .iter()
                        .filter(|p| matches_filter(p, state.filter))
                        .count();
                    ui.label(
                        RichText::new(format!("{count} results"))
                            .font(FontId::monospace(10.5))
                            .color(MUTED),
                    );
                });
            });
            ui.add_space(12.0);

            // Cards
            for p in state.proposals.iter().filter(|p| matches_filter(p, state.filter)) {
                if proposal_card(ui, p, false) {
                    state.selected = Some(p.id);
                }
                ui.add_space(10.0);
            }
        });
    nav
}

fn matches_filter(p: &Proposal, f: Filter) -> bool {
    match f {
        Filter::All => true,
        Filter::Pending => p.status == PStatus::Pending,
        Filter::Approved => p.status == PStatus::Approved,
    }
}

fn filter_chip(ui: &mut Ui, label: &str, active: bool) -> bool {
    let font = FontId::proportional(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let w = galley.size().x + 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg, stroke) = if active {
        (
            with_alpha(theme::PRIMARY, 28),
            theme::PRIMARY,
            Stroke::new(1.0, with_alpha(theme::PRIMARY, 110)),
        )
    } else if response.hovered() {
        (theme::SURFACE_HIGH, theme::TEXT, Stroke::new(1.0, theme::OUTLINE_VARIANT))
    } else {
        (theme::SURFACE_CONTAINER, SOFT, Stroke::new(1.0, theme::OUTLINE_VARIANT))
    };
    painter.rect(rect, 13.0, bg, stroke, StrokeKind::Inside);
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, fg);
    response.clicked()
}

// ----- proposal card -----

fn proposal_card(ui: &mut Ui, p: &Proposal, _compact: bool) -> bool {
    let response = egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER_LOW)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Header: kind + meeting + confidence
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(p.kind.label().to_uppercase())
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(p.kind.accent()),
                );
                if let Some(m) = p.meeting {
                    ui.label(
                        RichText::new(format!("from {m}"))
                            .font(FontId::monospace(10.0))
                            .color(MUTED),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}% confidence", p.confidence))
                            .font(FontId::monospace(10.5))
                            .color(SOFT),
                    );
                });
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(p.title)
                    .font(fonts::display(15.0))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(p.rationale).size(12.5).color(SOFT));

            ui.add_space(12.0);
            // Action buttons (decorative — focus mode is the real interaction).
            ui.horizontal(|ui| {
                action_btn(ui, icons::CHECK, "Accept", true, false);
                ui.add_space(6.0);
                action_btn(ui, icons::EDIT, "Refine", false, false);
                ui.add_space(6.0);
                action_btn(ui, icons::CLOSE, "Dismiss", false, true);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(p.generated)
                            .font(FontId::monospace(10.0))
                            .color(MUTED),
                    );
                });
            });
        })
        .response;
    let click =
        ui.interact(response.rect, response.id.with("card_click"), Sense::click());
    click.clicked()
}

fn action_btn(ui: &mut Ui, glyph: &str, label: &str, primary: bool, ghost: bool) -> bool {
    let font = FontId::proportional(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let w = 32.0 + galley.size().x;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
    let painter = ui.painter_at(rect);

    let (bg, fg, border) = if primary {
        (theme::PRIMARY, theme::BUTTON_TEXT, Stroke::new(1.0, theme::PRIMARY))
    } else if ghost {
        (
            if response.hovered() {
                theme::SURFACE_CONTAINER
            } else {
                Color32::TRANSPARENT
            },
            SOFT,
            Stroke::NONE,
        )
    } else {
        (
            if response.hovered() {
                theme::SURFACE_HIGH
            } else {
                theme::SURFACE_CONTAINER
            },
            theme::TEXT,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        )
    };
    painter.rect(rect, 3.0, bg, border, StrokeKind::Inside);
    painter.text(
        pos2(rect.left() + 10.0, rect.center().y),
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
    response.clicked()
}

// ----- focus view -----

fn show_focus(ui: &mut Ui, p: &Proposal, selected: &mut Option<&'static str>) -> Option<Open> {
    let mut nav: Option<Open> = None;
    egui::ScrollArea::vertical()
        .id_salt(("proposal_focus_scroll", p.id))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Back button
            if back_button(ui, "Back to Proposals") {
                *selected = None;
            }
            ui.add_space(12.0);

            // Meta row: kind + confidence bar + generated
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(p.kind.label().to_uppercase())
                        .font(FontId::monospace(10.5))
                        .strong()
                        .color(p.kind.accent()),
                );
                ui.add_space(12.0);
                confidence_bar(ui, p.confidence);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("generated {}", p.generated))
                            .font(FontId::monospace(10.5))
                            .color(MUTED),
                    );
                });
            });
            ui.add_space(8.0);

            // Title
            ui.label(
                RichText::new(p.title)
                    .font(fonts::display(24.0))
                    .strong()
                    .color(theme::TEXT),
            );

            // Source meeting
            if let Some(m) = p.meeting {
                ui.add_space(6.0);
                if source_meeting_link(ui, m) {
                    nav = Some(Open::Meeting(m.to_string()));
                }
            }

            ui.add_space(18.0);

            // Action buttons (large)
            ui.horizontal(|ui| {
                large_action(ui, icons::CHECK, "Accept proposal", ActionStyle::Primary);
                ui.add_space(8.0);
                large_action(ui, icons::EDIT, "Refine with AI", ActionStyle::Secondary);
                ui.add_space(8.0);
                large_action(ui, icons::CLOSE, "Dismiss", ActionStyle::Ghost);
            });

            ui.add_space(20.0);

            // Two-column grid
            let two_col = ui.available_width() >= FOCUS_GRID_BREAKPOINT;
            if two_col {
                let total = ui.available_width();
                let gap = 24.0;
                let left_w = ((total - gap) * 2.0 / 3.0).floor();
                let right_w = total - gap - left_w;
                ui.horizontal_top(|ui| {
                    let origin = ui.cursor().min;
                    let height = ui.available_height().max(360.0);
                    ui.allocate_rect(
                        Rect::from_min_size(origin, vec2(total, height)),
                        Sense::hover(),
                    );

                    let left_rect = Rect::from_min_size(origin, vec2(left_w, height));
                    let mut left = ui.new_child(
                        UiBuilder::new()
                            .max_rect(left_rect)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    if let Some(o) = focus_left(&mut left, p) {
                        nav = Some(o);
                    }

                    let right_rect = Rect::from_min_size(
                        pos2(origin.x + left_w + gap, origin.y),
                        vec2(right_w, height),
                    );
                    let mut right = ui.new_child(
                        UiBuilder::new()
                            .max_rect(right_rect)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    if let Some(o) = focus_right(&mut right, p) {
                        nav = Some(o);
                    }
                });
            } else {
                if let Some(o) = focus_left(ui, p) {
                    nav = Some(o);
                }
                ui.add_space(20.0);
                if let Some(o) = focus_right(ui, p) {
                    nav = Some(o);
                }
            }
        });
    nav
}

fn focus_left(ui: &mut Ui, p: &Proposal) -> Option<Open> {
    let mut nav: Option<Open> = None;

    section_header(ui, "RATIONALE");
    ui.add_space(4.0);
    ui.label(RichText::new(p.rationale).size(13.0).color(theme::TEXT));

    if let Some(d) = p.details {
        ui.add_space(20.0);
        section_header(ui, "DETAILS");
        ui.add_space(4.0);
        ui.label(RichText::new(d).size(13.0).color(theme::TEXT));
    }

    if !p.evidence.is_empty() {
        ui.add_space(20.0);
        section_header(ui, "EVIDENCE");
        ui.add_space(6.0);
        for e in &p.evidence {
            if let Some(o) = evidence_row(ui, e) {
                nav = Some(o);
            }
            ui.add_space(8.0);
        }
    }
    nav
}

fn focus_right(ui: &mut Ui, p: &Proposal) -> Option<Open> {
    let mut nav: Option<Open> = None;

    section_header(ui, "STATUS");
    ui.add_space(6.0);
    status_pill(ui, p.status);

    if !p.related_tasks.is_empty() {
        ui.add_space(20.0);
        section_header(ui, "RELATED TASKS");
        ui.add_space(6.0);
        for t in &p.related_tasks {
            if related_task_row(ui, t) {
                nav = Some(Open::Task(t.id.to_string()));
            }
            ui.add_space(4.0);
        }
    }

    if !p.related_docs.is_empty() {
        ui.add_space(20.0);
        section_header(ui, "RELATED DOCS");
        ui.add_space(6.0);
        for d in &p.related_docs {
            if related_doc_row(ui, d) {
                nav = Some(Open::Vault(d.vault_path.clone()));
            }
            ui.add_space(4.0);
        }
    }
    nav
}

// ----- focus helpers -----

fn back_button(ui: &mut Ui, label: &str) -> bool {
    let font = FontId::proportional(12.0);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), SOFT);
    let w = 26.0 + g.size().x;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
    let painter = ui.painter_at(rect);
    let fg = if response.hovered() { theme::TEXT } else { SOFT };
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        icons::ARROW_BACK,
        fonts::icon(14.0),
        fg,
    );
    painter.text(
        pos2(rect.left() + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        fg,
    );
    response.clicked()
}

fn confidence_bar(ui: &mut Ui, conf: u8) {
    let bar_w = 90.0;
    let bar_h = 4.0;
    let label_text = format!("{conf}% confidence");
    let label_font = FontId::monospace(10.5);
    let g = ui
        .painter()
        .layout_no_wrap(label_text.clone(), label_font.clone(), SOFT);
    let total_w = bar_w + 8.0 + g.size().x;
    let (rect, _) = ui.allocate_exact_size(vec2(total_w, 16.0), Sense::hover());
    let painter = ui.painter_at(rect);
    // Track
    let track = Rect::from_min_size(
        pos2(rect.left(), rect.center().y - bar_h / 2.0),
        vec2(bar_w, bar_h),
    );
    painter.rect_filled(track, 2.0, theme::SURFACE_HIGH);
    let fill_w = (bar_w * conf as f32 / 100.0).clamp(0.0, bar_w);
    painter.rect_filled(
        Rect::from_min_size(track.min, vec2(fill_w, bar_h)),
        2.0,
        theme::PRIMARY,
    );
    painter.text(
        pos2(rect.left() + bar_w + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        &label_text,
        label_font,
        SOFT,
    );
}

fn source_meeting_link(ui: &mut Ui, title: &str) -> bool {
    let label = format!("from {title}");
    let font = FontId::proportional(12.0);
    let g = ui.painter().layout_no_wrap(label.clone(), font.clone(), SOFT);
    let (rect, response) = ui.allocate_exact_size(vec2(g.size().x + 26.0, 22.0), Sense::click());
    let painter = ui.painter_at(rect);
    let color = if response.hovered() { theme::PRIMARY } else { SOFT };
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        icons::GRAPHIC_EQ,
        fonts::icon(13.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        &label,
        font,
        color,
    );
    response.clicked()
}

#[derive(Copy, Clone)]
enum ActionStyle {
    Primary,
    Secondary,
    Ghost,
}

fn large_action(ui: &mut Ui, glyph: &str, label: &str, style: ActionStyle) -> bool {
    let font = FontId::proportional(13.0);
    let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
    let w = 38.0 + g.size().x;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 38.0), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg, border) = match style {
        ActionStyle::Primary => (
            if response.hovered() {
                theme::PRIMARY_SOFT
            } else {
                theme::PRIMARY
            },
            theme::BUTTON_TEXT,
            Stroke::new(1.0, theme::PRIMARY),
        ),
        ActionStyle::Secondary => (
            if response.hovered() {
                theme::SURFACE_HIGH
            } else {
                theme::SURFACE_CONTAINER
            },
            theme::TEXT,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        ),
        ActionStyle::Ghost => (
            if response.hovered() {
                theme::SURFACE_CONTAINER
            } else {
                Color32::TRANSPARENT
            },
            SOFT,
            Stroke::NONE,
        ),
    };
    painter.rect(rect, 3.0, bg, border, StrokeKind::Inside);
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(15.0),
        fg,
    );
    painter.text(
        pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        fg,
    );
    response.clicked()
}

fn section_header(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .font(FontId::monospace(10.0))
            .strong()
            .color(MUTED),
    );
}

fn status_pill(ui: &mut Ui, status: PStatus) {
    let (text, color) = match status {
        PStatus::Pending => ("Pending review", AMBER),
        PStatus::Approved => ("Approved", theme::PRIMARY),
    };
    let font = FontId::proportional(11.5);
    let g = ui.painter().layout_no_wrap(text.to_string(), font.clone(), color);
    let w = g.size().x + 28.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        11.0,
        with_alpha(color, 24),
        Stroke::new(1.0, with_alpha(color, 110)),
        StrokeKind::Inside,
    );
    painter.circle_filled(pos2(rect.left() + 10.0, rect.center().y), 3.0, color);
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
}

fn related_task_row(ui: &mut Ui, t: &RelatedTask) -> bool {
    let h = 38.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let painter = ui.painter_at(rect);
    let stroke = if response.hovered() {
        Stroke::new(1.0, with_alpha(theme::PRIMARY, 120))
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };
    painter.rect(
        rect,
        2.0,
        if response.hovered() {
            theme::SURFACE_HIGH
        } else {
            theme::SURFACE_CONTAINER
        },
        stroke,
        StrokeKind::Inside,
    );
    painter.circle_filled(pos2(rect.left() + 12.0, rect.center().y), 3.5, t.status.dot());
    painter.text(
        pos2(rect.left() + 26.0, rect.center().y),
        Align2::LEFT_CENTER,
        t.id,
        FontId::monospace(10.5),
        theme::PRIMARY,
    );
    let title_font = FontId::proportional(11.5);
    let title_x = rect.left() + 90.0;
    painter.text(
        pos2(title_x, rect.center().y),
        Align2::LEFT_CENTER,
        t.title,
        title_font,
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(12.0),
        if response.hovered() { theme::PRIMARY } else { MUTED },
    );
    response.clicked()
}

fn related_doc_row(ui: &mut Ui, d: &RelatedDoc) -> bool {
    let h = 36.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let painter = ui.painter_at(rect);
    let stroke = if response.hovered() {
        Stroke::new(1.0, with_alpha(theme::PRIMARY, 120))
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };
    painter.rect(
        rect,
        2.0,
        if response.hovered() {
            theme::SURFACE_HIGH
        } else {
            theme::SURFACE_CONTAINER
        },
        stroke,
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::DESCRIPTION,
        fonts::icon(13.0),
        if response.hovered() { theme::PRIMARY } else { MUTED },
    );
    painter.text(
        pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        d.name,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 30.0, rect.center().y),
        Align2::RIGHT_CENTER,
        d.folder,
        FontId::monospace(10.0),
        MUTED,
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(12.0),
        if response.hovered() { theme::PRIMARY } else { MUTED },
    );
    response.clicked()
}

fn evidence_row(ui: &mut Ui, e: &Evidence) -> Option<Open> {
    let response = egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let glyph = match e.kind {
                    EvidenceKind::Transcript => icons::FORMAT_QUOTE,
                    EvidenceKind::Vault => icons::DESCRIPTION,
                };
                let (icon_rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                ui.painter().text(
                    icon_rect.center(),
                    Align2::CENTER_CENTER,
                    glyph,
                    fonts::icon(14.0),
                    MUTED,
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(e.meta)
                            .font(FontId::monospace(10.5))
                            .color(if matches!(e.kind, EvidenceKind::Transcript) {
                                e.accent
                            } else {
                                MUTED
                            }),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(e.text)
                            .size(12.5)
                            .italics()
                            .color(theme::TEXT),
                    );
                });
            });
        })
        .response;
    let click = ui.interact(response.rect, response.id.with("evidence_click"), Sense::click());
    if click.clicked() {
        return e.target.clone();
    }
    None
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

fn sample_proposals() -> Vec<Proposal> {
    let root = vault_root();
    let doc = |name: &'static str, folder: &'static str, rel: &str| RelatedDoc {
        name,
        folder,
        vault_path: root.join(rel),
    };

    vec![
        Proposal {
            id: "prop-chunking",
            kind: Kind::Solution,
            title: "Use 512-token windows with 64-token overlap as the RAG default",
            rationale:
                "Recall@10 peaks at 512/64 on the meetings corpus. Token budget stays within 18% \
                 of the 256-window baseline. Largest win on mid-length meetings (30–60 min).",
            details: Some(
                "Benchmarks show a 14% Recall@10 improvement at 512/64 vs the current 1024/128 \
                 default, with inference cost increasing only 6% because batching amortizes the \
                 overhead. Failure mode is long meetings (>90 min) where context fragmentation \
                 costs 3–4%. Recommended guardrail: auto-switch to 1024 windows for meetings over \
                 75 min.\n\nNext step if accepted: land as the default in src/rag/chunk.rs, gate \
                 behind EPIDOTE_RAG_CHUNK_STRATEGY=v2 for one week.",
            ),
            confidence: 82,
            status: PStatus::Pending,
            generated: "3 min ago",
            meeting: Some("System Architecture Sync"),
            evidence: vec![
                Evidence {
                    kind: EvidenceKind::Transcript,
                    text: "We discussed implementing DataLoader to batch those requests. Did we \
                           get a chance to prototype that on the staging environment in \
                           AWS-East-1?",
                    meta: "Sarah J. @ 01:22:04",
                    accent: theme::PRIMARY,
                    target: Some(Open::Meeting("System Architecture Sync".to_string())),
                },
                Evidence {
                    kind: EvidenceKind::Vault,
                    text: "Recall@10 peaks at 512/64 in the benchmark table.",
                    meta: "Chunking Benchmarks",
                    accent: SOFT,
                    target: Some(Open::Vault(
                        root.join("Architecture/Chunking Benchmarks.md"),
                    )),
                },
            ],
            related_tasks: vec![RelatedTask {
                id: "TSK-092",
                title: "Determine optimal chunking strategy",
                status: TaskStatus::Active,
            }],
            related_docs: vec![
                doc(
                    "Chunking Benchmarks",
                    "Architecture",
                    "Architecture/Chunking Benchmarks.md",
                ),
                doc(
                    "Embedding Architecture",
                    "Architecture",
                    "Architecture/Embedding Architecture.md",
                ),
            ],
        },
        Proposal {
            id: "prop-extract",
            kind: Kind::Task,
            title: "Create TSK: Benchmark DataLoader batching on GraphQL resolver",
            rationale:
                "Sarah mentioned staging on AWS-East-1 @ 01:23:48 and tied it to overnight perf \
                 monitoring. Currently no task captures this; tagging David as owner.",
            details: Some(
                "Proposed owner: David C. Proposed due: next Monday. Proposed parent meeting: \
                 System Architecture Sync. Baseline metric: p95 resolver latency on \
                 /meeting/:id/participants — currently 420ms.",
            ),
            confidence: 91,
            status: PStatus::Pending,
            generated: "1 min ago",
            meeting: Some("System Architecture Sync"),
            evidence: vec![Evidence {
                kind: EvidenceKind::Transcript,
                text: "If we push that update by end of day, we can monitor the performance \
                       metrics overnight.",
                meta: "Sarah J. @ 01:23:48",
                accent: theme::PRIMARY,
                target: Some(Open::Meeting("System Architecture Sync".to_string())),
            }],
            related_tasks: vec![],
            related_docs: vec![],
        },
        Proposal {
            id: "prop-roadmap",
            kind: Kind::Agenda,
            title: "Proposed agenda for Q4 Kickoff (next Tuesday)",
            rationale:
                "Based on Q3 roadmap carryover, 3 unresolved TSK items, and Priya's \
                 retrieval-quality theme. 6 items, estimated 52 min.",
            details: Some(
                "01. Retrieval quality review (Q3 metrics) — 10m\n\
                 02. TSK-092 chunking default · go/no-go — 8m\n\
                 03. TSK-118 roadmap one-pager walk-through — 12m\n\
                 04. Pentest Q3 remediation status — 7m\n\
                 05. Caching RFC v2 resourcing — 10m\n\
                 06. Open floor — 5m",
            ),
            confidence: 74,
            status: PStatus::Pending,
            generated: "2h ago",
            meeting: Some("Q3 Roadmap Planning"),
            evidence: vec![Evidence {
                kind: EvidenceKind::Vault,
                text: "Carryover items from Q3.",
                meta: "Q3 Roadmap v0",
                accent: SOFT,
                target: None,
            }],
            related_tasks: vec![
                RelatedTask {
                    id: "TSK-092",
                    title: "Determine optimal chunking strategy",
                    status: TaskStatus::Active,
                },
            ],
            related_docs: vec![],
        },
        Proposal {
            id: "prop-summary",
            kind: Kind::Summary,
            title: "Q3 Roadmap Planning — summary & decisions",
            rationale:
                "Q3 anchored on retrieval quality. TSK-092 (chunking defaults) is the critical \
                 dependency. Roadmap re-sequenced so the RAG pipeline cutover lands before the \
                 caching rewrite.",
            details: Some(
                "Decisions: (1) RAG pipeline cutover moves to week 2 of Q3. (2) Caching rewrite \
                 pushed one sprint. (3) Pentest remediation stays in Q3 scope.\n\nOpen \
                 questions: ownership of the staged rollout, and whether the eval suite needs a \
                 parallel run before cutover.",
            ),
            confidence: 88,
            status: PStatus::Approved,
            generated: "5h ago",
            meeting: Some("Q3 Roadmap Planning"),
            evidence: vec![],
            related_tasks: vec![RelatedTask {
                id: "TSK-092",
                title: "Determine optimal chunking strategy",
                status: TaskStatus::Active,
            }],
            related_docs: vec![],
        },
        Proposal {
            id: "prop-doc-migration",
            kind: Kind::Doc,
            title: "Generated: Migration Manifesto v1.md",
            rationale:
                "Full architectural proposal extracted from the meeting transcript. 4 sections, \
                 references 2 existing vault notes.",
            details: Some(
                "Sections: Phase 1 (schema mapping · Marcus), Phase 2 (event bus · Sarah), \
                 Phase 3 (monolith decomposition · David), Risks. Cross-links to Embedding \
                 Architecture and Caching RFC v2.",
            ),
            confidence: 79,
            status: PStatus::Pending,
            generated: "2d ago",
            meeting: Some("Q3 Data Architecture"),
            evidence: vec![Evidence {
                kind: EvidenceKind::Vault,
                text: "Draft auto-saved to the Vault under Architecture/.",
                meta: "Migration Manifesto v1",
                accent: SOFT,
                target: Some(Open::Vault(
                    root.join("Architecture/Caching RFC v2.md"),
                )),
            }],
            related_tasks: vec![RelatedTask {
                id: "TSK-104",
                title: "Review Redis caching strategy",
                status: TaskStatus::Triage,
            }],
            related_docs: vec![doc(
                "Embedding Architecture",
                "Architecture",
                "Architecture/Embedding Architecture.md",
            )],
        },
        Proposal {
            id: "prop-caching-owner",
            kind: Kind::Task,
            title: "Create TSK: Nominate owner for Caching RFC v2 resourcing",
            rationale:
                "Caching rewrite was re-sequenced but no owner was explicitly assigned in the \
                 meeting. Auto-routing to the on-call architect.",
            details: Some(
                "Proposed owner: Marcus L. Proposed due: end of Q3 week 3.",
            ),
            confidence: 68,
            status: PStatus::Pending,
            generated: "2d ago",
            meeting: Some("Q3 Data Architecture"),
            evidence: vec![Evidence {
                kind: EvidenceKind::Vault,
                text: "RFC v2 lacks an assigned owner field.",
                meta: "Caching RFC v2",
                accent: SOFT,
                target: Some(Open::Vault(root.join("Architecture/Caching RFC v2.md"))),
            }],
            related_tasks: vec![RelatedTask {
                id: "TSK-104",
                title: "Review Redis caching strategy",
                status: TaskStatus::Triage,
            }],
            related_docs: vec![doc(
                "Caching RFC v2",
                "Architecture",
                "Architecture/Caching RFC v2.md",
            )],
        },
    ]
}
