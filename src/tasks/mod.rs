use std::path::PathBuf;

use chrono::NaiveDate;
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Context, FontId, Id, Layout, Margin, Order, Rect,
    RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder,
};

use crate::{fonts, icons, nav::Open, theme};

const COLUMN_W: f32 = 320.0;
const COLUMN_GAP: f32 = 16.0;
const CARD_GAP: f32 = 12.0;
const ERROR: Color32 = Color32::from_rgb(0xff, 0xb4, 0xab);

// ----- public state -----

pub struct State {
    columns: Vec<Column>,
    selected: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            columns: sample_columns(),
            selected: None,
        }
    }
}

impl State {
    /// Select a task by ID, showing its detail popup on next render. If the ID
    /// doesn't exist, the selection is cleared.
    pub fn select(&mut self, id: String) {
        if find_task(&self.columns, &id).is_some() {
            self.selected = Some(id);
        } else {
            self.selected = None;
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    show_header(ui);
    ui.add_space(20.0);

    let mut clicked: Option<&'static str> = None;
    show_board(ui, &state.columns, state.selected.as_deref(), &mut clicked);
    if let Some(id) = clicked {
        state.selected = Some(id.to_string());
    }

    let mut nav: Option<Open> = None;
    let current = state.selected.clone();
    if let Some(id) = current {
        if let Some((task, status)) = find_task(&state.columns, &id) {
            let mut keep: Option<String> = Some(id);
            if let Some(o) = show_task_drawer(ui.ctx(), task, status, &mut keep) {
                nav = Some(o);
            }
            state.selected = keep;
        } else {
            state.selected = None;
        }
    }
    nav
}

fn find_task<'a>(columns: &'a [Column], id: &str) -> Option<(&'a Task, Status)> {
    for col in columns {
        for task in &col.tasks {
            if task.id == id {
                return Some((task, col.status));
            }
        }
    }
    None
}

// ----- header -----

fn show_header(ui: &mut Ui) {
    let header_h = 56.0;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), header_h),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Tasks Board")
                        .font(fonts::display(28.0))
                        .strong()
                        .color(theme::TEXT),
                );
                ui.add_space(-2.0);
                ui.label(
                    RichText::new(
                        "Extracted technical action items from recent syncs and architectural reviews.",
                    )
                    .size(12.5)
                    .color(theme::DIM_TEXT),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                filter_button(ui);
            });
        },
    );
}

fn filter_button(ui: &mut Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(96.0, 32.0), Sense::click());
    let painter = ui.painter_at(rect);
    let bg = if response.hovered() {
        theme::SURFACE_CONTAINER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect(
        rect,
        2.0,
        bg,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::FILTER_LIST,
        fonts::icon(16.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Filter",
        FontId::proportional(12.0),
        theme::TEXT,
    );
    response
}

// ----- domain types -----

#[derive(Clone, Copy)]
enum Status {
    Triage,
    Active,
    Blocked,
}

impl Status {
    fn dot_color(self) -> Color32 {
        match self {
            Status::Triage => theme::OUTLINE,
            Status::Active => theme::PRIMARY,
            Status::Blocked => ERROR,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Status::Triage => "Triage",
            Status::Active => "Active Analysis",
            Status::Blocked => "Blocked",
        }
    }
}

#[derive(Clone, Copy)]
enum VaultKind {
    Proposal,
    Document,
    Epic,
}

impl VaultKind {
    fn glyph(self) -> &'static str {
        match self {
            VaultKind::Proposal => icons::DESCRIPTION,
            VaultKind::Document => icons::INVENTORY_2,
            VaultKind::Epic => icons::TASK,
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            VaultKind::Proposal => "Prop",
            VaultKind::Document => "Doc",
            VaultKind::Epic => "Epic",
        }
    }
}

struct VaultLink {
    kind: VaultKind,
    label: &'static str,
    vault_path: Option<PathBuf>,
}

struct Task {
    id: &'static str,
    title: &'static str,
    due_date: Option<NaiveDate>,
    meeting: Option<&'static str>,
    vault_links: Vec<VaultLink>,
    description: Option<&'static str>,
    blocked_note: Option<&'static str>,
    proposal_id: Option<&'static str>,
}

struct Column {
    name: &'static str,
    status: Status,
    tasks: Vec<Task>,
}

// ----- board layout -----

fn show_board(
    ui: &mut Ui,
    columns: &[Column],
    selected_id: Option<&str>,
    clicked: &mut Option<&'static str>,
) {
    let board_h = ui.available_height();

    egui::ScrollArea::horizontal()
        .id_salt("tasks_board_h")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (i, col) in columns.iter().enumerate() {
                    show_column(ui, col, board_h, selected_id, clicked);
                    if i + 1 < columns.len() {
                        ui.add_space(COLUMN_GAP);
                    }
                }
            });
        });
}

fn show_column(
    ui: &mut Ui,
    column: &Column,
    height: f32,
    selected_id: Option<&str>,
    clicked: &mut Option<&'static str>,
) {
    let (col_rect, _) = ui.allocate_exact_size(vec2(COLUMN_W, height), Sense::hover());
    ui.painter()
        .rect_filled(col_rect, 4.0, theme::SURFACE_CONTAINER_LOW);

    let inner_rect = col_rect.shrink(16.0);
    let mut inner = ui.new_child(
        UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(Align::Min)),
    );

    show_column_header(&mut inner, column);
    inner.add_space(14.0);

    egui::ScrollArea::vertical()
        .id_salt(("tasks_col_v", column.name))
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut inner, |ui| {
            ui.set_width(inner_rect.width());
            for (i, task) in column.tasks.iter().enumerate() {
                let is_selected = selected_id == Some(task.id);
                if show_task_card(ui, task, is_selected) {
                    *clicked = Some(task.id);
                }
                if i + 1 < column.tasks.len() {
                    ui.add_space(CARD_GAP);
                }
            }
        });
}

fn show_column_header(ui: &mut Ui, column: &Column) {
    ui.horizontal(|ui| {
        let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
        ui.painter()
            .circle_filled(dot_rect.center(), 4.0, column.status.dot_color());
        ui.add_space(2.0);
        ui.label(
            RichText::new(column.name)
                .font(fonts::display(12.0))
                .strong()
                .color(theme::TEXT),
        );
        ui.label(
            RichText::new(format!("{}", column.tasks.len()))
                .size(11.5)
                .color(theme::DIM_TEXT),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                icons::MORE_HORIZ,
                fonts::icon(16.0),
                theme::DIM_TEXT,
            );
        });
    });
}

// ----- card -----

fn show_task_card(ui: &mut Ui, task: &Task, is_selected: bool) -> bool {
    let bg = if is_selected {
        theme::SURFACE_HIGHEST
    } else {
        theme::SURFACE_CONTAINER
    };
    let stroke = if is_selected {
        Stroke::new(2.0, theme::PRIMARY)
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };

    let inner = egui::Frame::default()
        .fill(bg)
        .stroke(stroke)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                show_id_pill(ui, task);
            });
            ui.add_space(8.0);

            ui.label(RichText::new(task.title).size(13.0).color(theme::TEXT));

            if let Some(note) = task.blocked_note {
                ui.add_space(8.0);
                show_blocked_note(ui, note);
            }

            let has_sources = task.meeting.is_some()
                || !task.vault_links.is_empty()
                || task.proposal_id.is_some();
            if has_sources {
                ui.add_space(10.0);
                let mut first = true;
                if let Some(meeting) = task.meeting {
                    show_source_row(
                        ui,
                        icons::GRAPHIC_EQ,
                        &format!("Sync: {meeting}"),
                        theme::DIM_TEXT,
                    );
                    first = false;
                }
                if task.proposal_id.is_some() {
                    if !first {
                        ui.add_space(2.0);
                    }
                    show_source_row(
                        ui,
                        icons::BOLT,
                        "from AI proposal",
                        theme::ACCENT_AMBER,
                    );
                    first = false;
                }
                for link in &task.vault_links {
                    if !first {
                        ui.add_space(2.0);
                    }
                    show_source_row(
                        ui,
                        link.kind.glyph(),
                        &format!("{}: {}", link.kind.prefix(), link.label),
                        theme::DIM_TEXT,
                    );
                    first = false;
                }
            }
        });

    let response = ui.interact(
        inner.response.rect,
        inner.response.id.with("card_click"),
        Sense::click(),
    );
    response.clicked()
}

fn show_id_pill(ui: &mut Ui, task: &Task) {
    let (color, bg, border) = if task.blocked_note.is_some() {
        (
            ERROR,
            Color32::TRANSPARENT,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xb4, 0xab, 90)),
        )
    } else {
        (theme::DIM_TEXT, theme::SURFACE_CONTAINER_LOW, Stroke::NONE)
    };

    let font = FontId::monospace(10.5);
    let galley = ui
        .painter()
        .layout_no_wrap(task.id.to_string(), font.clone(), color);
    let pad_x = 8.0;
    let height = 18.0;
    let width = galley.size().x + pad_x * 2.0;

    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(rect, 2.0, bg, border, StrokeKind::Inside);
    painter.text(rect.center(), Align2::CENTER_CENTER, task.id, font, color);
}

fn show_source_row(ui: &mut Ui, glyph: &str, label: &str, icon_color: Color32) {
    let text_color = theme::DIM_TEXT;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(13.0),
        icon_color,
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        text_color,
    );
}

fn show_blocked_note(ui: &mut Ui, note: &str) {
    egui::Frame::default()
        .fill(Color32::from_rgba_unmultiplied(0xff, 0xb4, 0xab, 22))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(0xff, 0xb4, 0xab, 60),
        ))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.label(RichText::new(note).size(11.0).color(ERROR));
        });
}

// ----- detail drawer (slide-from-right) -----

const DRAWER_W: f32 = 440.0;
const TITLE_BAR_H: f32 = 32.0;

fn show_task_drawer(
    ctx: &Context,
    task: &Task,
    status: Status,
    selected: &mut Option<String>,
) -> Option<Open> {
    let mut nav: Option<Open> = None;

    // Backdrop — dim everything below the title bar; click to close.
    egui::Area::new(Id::new(("task_drawer_backdrop", task.id)))
        .order(Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let backdrop = Rect::from_min_max(
                pos2(screen.left(), screen.top() + TITLE_BAR_H),
                screen.right_bottom(),
            );
            let resp = ui.allocate_rect(backdrop, Sense::click());
            ui.painter().rect_filled(
                backdrop,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 110),
            );
            if resp.clicked() {
                *selected = None;
            }
        });

    // Drawer panel — anchored to the right edge, full height below the title bar.
    egui::Area::new(Id::new(("task_drawer", task.id)))
        .order(Order::Tooltip)
        .anchor(Align2::RIGHT_TOP, [0.0, TITLE_BAR_H])
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let h = (screen.height() - TITLE_BAR_H).max(200.0);

            egui::Frame::default()
                .fill(theme::SURFACE_CONTAINER)
                .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
                .inner_margin(Margin::same(0))
                .show(ui, |ui| {
                    ui.set_width(DRAWER_W);
                    ui.set_min_height(h);
                    ui.set_max_height(h);

                    // Header — natural layout inside a frame so the status pill
                    // and inline due can wrap to a second line if needed without
                    // overlapping the separator below.
                    let header_frame = egui::Frame::default()
                        .inner_margin(Margin {
                            left: 18,
                            right: 18,
                            top: 16,
                            bottom: 14,
                        })
                        .show(ui, |ui| {
                            ui.set_width(DRAWER_W - 36.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(task.id)
                                        .font(FontId::monospace(11.5))
                                        .color(theme::DIM_TEXT),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let (close_rect, close_resp) = ui
                                        .allocate_exact_size(vec2(24.0, 24.0), Sense::click());
                                    let p = ui.painter_at(close_rect);
                                    if close_resp.hovered() {
                                        p.rect_filled(close_rect, 3.0, theme::SURFACE_HIGH);
                                    }
                                    p.text(
                                        close_rect.center(),
                                        Align2::CENTER_CENTER,
                                        icons::CLOSE,
                                        fonts::icon(14.0),
                                        if close_resp.hovered() {
                                            theme::TEXT
                                        } else {
                                            theme::DIM_TEXT
                                        },
                                    );
                                    if close_resp.clicked() {
                                        *selected = None;
                                    }
                                });
                            });
                            ui.add_space(6.0);
                            // Title wraps to multiple lines when long.
                            ui.label(
                                RichText::new(task.title)
                                    .font(fonts::display(20.0))
                                    .strong()
                                    .color(theme::TEXT),
                            );
                            ui.add_space(12.0);
                            // Pill + due — horizontal_wrapped so they flow to a
                            // second row on narrow widths instead of overlapping.
                            ui.horizontal_wrapped(|ui| {
                                show_status_badge(ui, status);
                                if let Some(due) = task.due_date {
                                    ui.add_space(10.0);
                                    show_due_inline(ui, due);
                                }
                            });
                        });
                    // Separator sits right below the header frame, no overlap.
                    let hr_y = header_frame.response.rect.bottom();
                    ui.painter().line_segment(
                        [
                            pos2(header_frame.response.rect.left(), hr_y),
                            pos2(header_frame.response.rect.right(), hr_y),
                        ],
                        Stroke::new(1.0, theme::OUTLINE_VARIANT),
                    );

                    // Body — natural layout inside a ScrollArea.
                    egui::ScrollArea::vertical()
                        .id_salt(("task_drawer_body", task.id))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Frame::default()
                                .inner_margin(Margin {
                                    left: 18,
                                    right: 18,
                                    top: 14,
                                    bottom: 18,
                                })
                                .show(ui, |ui| {
                                    ui.set_width(DRAWER_W - 36.0);
                                    if let Some(o) = drawer_body(ui, task) {
                                        nav = Some(o);
                                    }
                                });
                        });
                });
        });

    nav
}

fn drawer_body(ui: &mut Ui, task: &Task) -> Option<Open> {
    let mut nav: Option<Open> = None;

    if let Some(note) = task.blocked_note {
        section_header(ui, "BLOCKED");
        show_blocked_note(ui, note);
        ui.add_space(16.0);
    }

    if let Some(desc) = task.description {
        section_header(ui, "DETAILS");
        ui.label(RichText::new(desc).size(12.5).color(theme::TEXT));
        ui.add_space(16.0);
    }

    if let Some(meeting) = task.meeting {
        section_header(ui, "SOURCE MEETING");
        if drawer_link_row(ui, icons::GRAPHIC_EQ, meeting, "open", theme::DIM_TEXT) {
            nav = Some(Open::Meeting(meeting.to_string()));
        }
        ui.add_space(16.0);
    }

    if let Some(prop_id) = task.proposal_id {
        section_header(ui, "ORIGINATING PROPOSAL");
        if drawer_link_row(
            ui,
            icons::BOLT,
            "Open in proposals",
            "review",
            theme::ACCENT_AMBER,
        ) {
            nav = Some(Open::Proposal(prop_id.to_string()));
        }
        ui.add_space(16.0);
    }

    if !task.vault_links.is_empty() {
        section_header(ui, "VAULT REFERENCES");
        for link in &task.vault_links {
            let glyph = link.kind.glyph();
            let label = link.label;
            let folder = link.kind.prefix();
            if drawer_link_row(ui, glyph, label, folder, theme::DIM_TEXT) {
                if let Some(p) = &link.vault_path {
                    nav = Some(Open::Vault(p.clone()));
                }
            }
            ui.add_space(6.0);
        }
        ui.add_space(10.0);
    }

    nav
}

fn show_due_inline(ui: &mut Ui, date: NaiveDate) {
    let today = chrono::Local::now().date_naive();
    let days = (date - today).num_days();
    let color = match days {
        d if d < 0 => ERROR,
        d if d <= 3 => theme::PRIMARY,
        _ => theme::DIM_TEXT,
    };
    let label = format!("Due {}", date.format("%b %-d"));
    ui.label(
        RichText::new(label)
            .font(FontId::monospace(11.0))
            .color(color),
    );
}

fn drawer_link_row(
    ui: &mut Ui,
    glyph: &str,
    label: &str,
    trailing: &str,
    icon_color: Color32,
) -> bool {
    let h = 36.0;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let painter = ui.painter_at(rect);
    let stroke = if response.hovered() {
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 136, 110))
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };
    let bg = if response.hovered() {
        theme::SURFACE_HIGH
    } else {
        theme::SURFACE_CONTAINER_LOW
    };
    painter.rect(rect, 2.0, bg, stroke, StrokeKind::Inside);
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(14.0),
        icon_color,
    );
    painter.text(
        pos2(rect.left() + 34.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 30.0, rect.center().y),
        Align2::RIGHT_CENTER,
        trailing,
        FontId::monospace(10.0),
        theme::DIM_TEXT,
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::OPEN_IN_NEW,
        fonts::icon(11.0),
        if response.hovered() {
            theme::PRIMARY
        } else {
            theme::DIM_TEXT
        },
    );
    response.clicked()
}

fn section_header(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .font(fonts::display(10.5))
            .strong()
            .color(theme::DIM_TEXT),
    );
    ui.add_space(6.0);
}

fn show_status_badge(ui: &mut Ui, status: Status) {
    let color = status.dot_color();
    let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30);
    let label = status.label();
    let font = FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font.clone(), color);

    let pad_x = 10.0;
    let pad_dot = 16.0;
    let h = 22.0;
    let w = galley.size().x + pad_dot + pad_x;

    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(rect, 2.0, bg, Stroke::new(1.0, color), StrokeKind::Inside);
    painter.circle_filled(pos2(rect.left() + 8.0, rect.center().y), 3.0, color);
    painter.text(
        pos2(rect.left() + pad_dot, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        color,
    );
}


// ----- sample data -----

fn vault_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Documents/Epidote/Vault")
}

fn sample_columns() -> Vec<Column> {
    let today = chrono::Local::now().date_naive();
    let due = |offset: i64| today + chrono::Duration::days(offset);
    let root = vault_root();
    let path = |rel: &str| Some(root.join(rel));

    vec![
        Column {
            name: "TRIAGE",
            status: Status::Triage,
            tasks: vec![
                Task {
                    id: "TSK-104",
                    title: "Review Redis caching strategy for high-frequency reads",
                    due_date: Some(due(9)),
                    meeting: Some("Q3 Data Architecture"),
                    vault_links: vec![VaultLink {
                        kind: VaultKind::Proposal,
                        label: "Caching RFC v2",
                        vault_path: path("Architecture/Caching RFC v2.md"),
                    }],
                    description: Some(
                        "Cache hit rate on hot keys is plateauing around 78%. Compare \
                         current per-request lookup against a write-through layer with \
                         TTL bucketing, and call out memory ceiling implications.",
                    ),
                    blocked_note: None,
                    proposal_id: Some("prop-caching-owner"),
                },
                Task {
                    id: "TSK-108",
                    title: "Update IAM roles for new microservices deployment",
                    due_date: None,
                    meeting: None,
                    vault_links: vec![VaultLink {
                        kind: VaultKind::Proposal,
                        label: "Auth V2",
                        vault_path: None,
                    }],
                    description: Some(
                        "Three new services (graph-indexer, vault-search, transcript-rag) \
                         need scoped roles with least-privilege defaults.",
                    ),
                    blocked_note: None,
                    proposal_id: None,
                },
                Task {
                    id: "TSK-112",
                    title: "Spec retry/backoff policy for ingest workers",
                    due_date: Some(due(14)),
                    meeting: Some("Pipeline Reliability"),
                    vault_links: vec![],
                    description: None,
                    blocked_note: None,
                    proposal_id: None,
                },
            ],
        },
        Column {
            name: "ACTIVE ANALYSIS",
            status: Status::Active,
            tasks: vec![
                Task {
                    id: "TSK-092",
                    title: "Determine optimal chunking strategy for vector embeddings",
                    due_date: Some(due(4)),
                    meeting: Some("System Architecture Sync"),
                    vault_links: vec![
                        VaultLink {
                            kind: VaultKind::Document,
                            label: "Embedding Architecture",
                            vault_path: path("Architecture/Embedding Architecture.md"),
                        },
                        VaultLink {
                            kind: VaultKind::Document,
                            label: "Chunking Benchmarks",
                            vault_path: path("Architecture/Chunking Benchmarks.md"),
                        },
                    ],
                    description: Some(
                        "Trade-off between recall and token budget. Run side-by-side \
                         eval on the meetings corpus with 256 / 512 / 1024-token windows \
                         and 0 / 64 / 128-token overlap. Land on a default before the RAG \
                         cutover.",
                    ),
                    blocked_note: None,
                    proposal_id: Some("prop-chunking"),
                },
                Task {
                    id: "TSK-118",
                    title: "Publish Q3 roadmap one-pager",
                    due_date: Some(due(2)),
                    meeting: Some("Q3 Roadmap Planning"),
                    vault_links: vec![VaultLink {
                        kind: VaultKind::Document,
                        label: "Q3 Roadmap v0",
                        vault_path: None,
                    }],
                    description: Some(
                        "Anchor Q3 around retrieval quality. Re-sequence so the RAG pipeline \
                         cutover lands before the caching rewrite. Pentest remediation stays \
                         in scope.",
                    ),
                    blocked_note: None,
                    proposal_id: Some("prop-summary"),
                },
                Task {
                    id: "TSK-096",
                    title: "Profile cold-start latency on inference workers",
                    due_date: None,
                    meeting: Some("Latency Sweep"),
                    vault_links: vec![],
                    description: Some(
                        "First-token latency spikes after autoscale-down events. \
                         Capture flame graphs from the next idle cycle.",
                    ),
                    blocked_note: None,
                    proposal_id: None,
                },
            ],
        },
        Column {
            name: "BLOCKED",
            status: Status::Blocked,
            tasks: vec![Task {
                id: "TSK-088",
                title: "Migrate legacy logging to Datadog cluster",
                due_date: Some(due(1)),
                meeting: Some("Observability Overhaul"),
                vault_links: vec![VaultLink {
                    kind: VaultKind::Epic,
                    label: "Observability Overhaul",
                    vault_path: None,
                }],
                description: Some(
                    "Agent rollout staged behind security review. Once the policy bundle \
                     is signed off, switch the legacy syslog forwarder to the Datadog \
                     agent in two phases.",
                ),
                blocked_note: Some("Awaiting security approval for agent deployment."),
                proposal_id: None,
            }],
        },
    ]
}
