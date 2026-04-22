use chrono::NaiveDate;
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Context, FontId, Layout, Margin, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder,
};

use crate::{fonts, icons, theme};

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

pub fn show(ui: &mut Ui, state: &mut State) {
    show_header(ui);
    ui.add_space(20.0);

    let mut clicked: Option<&'static str> = None;
    show_board(ui, &state.columns, &mut clicked);
    if let Some(id) = clicked {
        state.selected = Some(id.to_string());
    }

    let current = state.selected.clone();
    if let Some(id) = current {
        if let Some((task, status)) = find_task(&state.columns, &id) {
            let mut keep: Option<String> = Some(id);
            show_task_popup(ui.ctx(), task, status, &mut keep);
            state.selected = keep;
        } else {
            state.selected = None;
        }
    }
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
}

struct Task {
    id: &'static str,
    title: &'static str,
    is_active: bool,
    due_date: Option<NaiveDate>,
    meeting: Option<&'static str>,
    vault_links: Vec<VaultLink>,
    description: Option<&'static str>,
    blocked_note: Option<&'static str>,
}

struct Column {
    name: &'static str,
    status: Status,
    tasks: Vec<Task>,
}

// ----- board layout -----

fn show_board(ui: &mut Ui, columns: &[Column], clicked: &mut Option<&'static str>) {
    let board_h = ui.available_height();

    egui::ScrollArea::horizontal()
        .id_salt("tasks_board_h")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (i, col) in columns.iter().enumerate() {
                    show_column(ui, col, board_h, clicked);
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
                if show_task_card(ui, task) {
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

fn show_task_card(ui: &mut Ui, task: &Task) -> bool {
    let bg = if task.is_active {
        theme::SURFACE_HIGHEST
    } else {
        theme::SURFACE_CONTAINER
    };
    let stroke = if task.is_active {
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 136, 110))
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

            let has_sources = task.meeting.is_some() || !task.vault_links.is_empty();
            if has_sources {
                ui.add_space(10.0);
                if task.is_active {
                    let p = ui.cursor().min;
                    let w = ui.available_width();
                    ui.painter().line_segment(
                        [p, pos2(p.x + w, p.y)],
                        Stroke::new(1.0, theme::OUTLINE_VARIANT),
                    );
                    ui.add_space(8.0);
                }
                let mut first = true;
                if let Some(meeting) = task.meeting {
                    show_source_row(
                        ui,
                        icons::VIDEO_LIBRARY,
                        &format!("Sync: {meeting}"),
                        task.is_active,
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
                        task.is_active,
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
    } else if task.is_active {
        (
            theme::PRIMARY,
            Color32::from_rgba_unmultiplied(0, 255, 136, 30),
            Stroke::NONE,
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

fn show_source_row(ui: &mut Ui, glyph: &str, label: &str, active: bool) {
    let color = if active { theme::TEXT } else { theme::DIM_TEXT };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(13.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        color,
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

// ----- detail popup -----

fn show_task_popup(
    ctx: &Context,
    task: &Task,
    status: Status,
    selected: &mut Option<String>,
) {
    let mut open = true;

    egui::Window::new(
        RichText::new(task.id)
            .font(FontId::monospace(11.5))
            .color(theme::DIM_TEXT),
    )
    .id(egui::Id::new(("task_popup", task.id)))
    .open(&mut open)
    .collapsible(false)
    .resizable(false)
    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
    .frame(
        egui::Frame::window(&ctx.style())
            .fill(theme::SURFACE_CONTAINER)
            .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT)),
    )
    .show(ctx, |ui| {
        ui.set_min_width(440.0);
        ui.set_max_width(520.0);

        ui.label(
            RichText::new(task.title)
                .font(fonts::display(20.0))
                .strong()
                .color(theme::TEXT),
        );
        ui.add_space(8.0);
        show_status_badge(ui, status);

        ui.add_space(18.0);

        if let Some(due) = task.due_date {
            section_header(ui, "DUE");
            show_due_row(ui, due);
            ui.add_space(14.0);
        }

        if let Some(meeting) = task.meeting {
            section_header(ui, "FROM MEETING");
            show_meta_row(ui, icons::VIDEO_LIBRARY, meeting, theme::TEXT);
            ui.add_space(14.0);
        }

        if !task.vault_links.is_empty() {
            section_header(ui, "VAULT REFERENCES");
            for link in &task.vault_links {
                show_vault_link_row(ui, link);
                ui.add_space(6.0);
            }
            ui.add_space(8.0);
        }

        if let Some(note) = task.blocked_note {
            section_header(ui, "BLOCKED");
            show_blocked_note(ui, note);
            ui.add_space(14.0);
        }

        if let Some(desc) = task.description {
            section_header(ui, "DETAILS");
            ui.label(
                RichText::new(desc)
                    .size(12.5)
                    .color(theme::TEXT),
            );
        }
    });

    if !open {
        *selected = None;
    }
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

fn show_due_row(ui: &mut Ui, date: NaiveDate) {
    let today = chrono::Local::now().date_naive();
    let days = (date - today).num_days();
    let (relative, color) = match days {
        d if d < 0 => (
            format!("{} day{} overdue", -d, if -d == 1 { "" } else { "s" }),
            ERROR,
        ),
        0 => ("Today".to_string(), theme::PRIMARY),
        1 => ("Tomorrow".to_string(), theme::PRIMARY),
        d if d <= 3 => (format!("in {d} days"), theme::PRIMARY),
        d => (format!("in {d} days"), theme::TEXT),
    };
    let label = format!("{}  ·  {}", date.format("%b %-d, %Y"), relative);
    show_meta_row(ui, icons::CALENDAR_TODAY, &label, color);
}

fn show_meta_row(ui: &mut Ui, glyph: &str, label: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        fonts::icon(16.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 26.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        color,
    );
}

fn show_vault_link_row(ui: &mut Ui, link: &VaultLink) {
    let height = 40.0;
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), height),
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
        2.0,
        bg,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        link.kind.glyph(),
        fonts::icon(16.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 40.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        link.label,
        FontId::proportional(12.5),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 40.0, rect.top() + 24.0),
        Align2::LEFT_TOP,
        link.kind.prefix(),
        FontId::proportional(10.0),
        theme::DIM_TEXT,
    );
}

// ----- sample data -----

fn sample_columns() -> Vec<Column> {
    let today = chrono::Local::now().date_naive();
    let due = |offset: i64| today + chrono::Duration::days(offset);

    vec![
        Column {
            name: "TRIAGE",
            status: Status::Triage,
            tasks: vec![
                Task {
                    id: "TSK-104",
                    title: "Review Redis caching strategy for high-frequency reads",
                    is_active: false,
                    due_date: Some(due(9)),
                    meeting: Some("Q3 Data Architecture"),
                    vault_links: vec![VaultLink {
                        kind: VaultKind::Proposal,
                        label: "Caching RFC v2",
                    }],
                    description: Some(
                        "Cache hit rate on hot keys is plateauing around 78%. Compare \
                         current per-request lookup against a write-through layer with \
                         TTL bucketing, and call out memory ceiling implications.",
                    ),
                    blocked_note: None,
                },
                Task {
                    id: "TSK-108",
                    title: "Update IAM roles for new microservices deployment",
                    is_active: false,
                    due_date: None,
                    meeting: None,
                    vault_links: vec![VaultLink {
                        kind: VaultKind::Proposal,
                        label: "Auth V2",
                    }],
                    description: Some(
                        "Three new services (graph-indexer, vault-search, transcript-rag) \
                         need scoped roles with least-privilege defaults.",
                    ),
                    blocked_note: None,
                },
                Task {
                    id: "TSK-112",
                    title: "Spec retry/backoff policy for ingest workers",
                    is_active: false,
                    due_date: Some(due(14)),
                    meeting: Some("Pipeline Reliability"),
                    vault_links: vec![],
                    description: None,
                    blocked_note: None,
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
                    is_active: true,
                    due_date: Some(due(4)),
                    meeting: Some("RAG Pipeline Optimization"),
                    vault_links: vec![
                        VaultLink {
                            kind: VaultKind::Document,
                            label: "Embedding Architecture",
                        },
                        VaultLink {
                            kind: VaultKind::Document,
                            label: "Chunking Benchmarks",
                        },
                    ],
                    description: Some(
                        "Trade-off between recall and token budget. Run side-by-side \
                         eval on the meetings corpus with 256 / 512 / 1024-token windows \
                         and 0 / 64 / 128-token overlap. Land on a default before the RAG \
                         cutover.",
                    ),
                    blocked_note: None,
                },
                Task {
                    id: "TSK-096",
                    title: "Profile cold-start latency on inference workers",
                    is_active: false,
                    due_date: None,
                    meeting: Some("Latency Sweep"),
                    vault_links: vec![],
                    description: Some(
                        "First-token latency spikes after autoscale-down events. \
                         Capture flame graphs from the next idle cycle.",
                    ),
                    blocked_note: None,
                },
            ],
        },
        Column {
            name: "BLOCKED",
            status: Status::Blocked,
            tasks: vec![Task {
                id: "TSK-088",
                title: "Migrate legacy logging to Datadog cluster",
                is_active: false,
                due_date: Some(due(1)),
                meeting: Some("Observability Overhaul"),
                vault_links: vec![VaultLink {
                    kind: VaultKind::Epic,
                    label: "Observability Overhaul",
                }],
                description: Some(
                    "Agent rollout staged behind security review. Once the policy bundle \
                     is signed off, switch the legacy syslog forwarder to the Datadog \
                     agent in two phases.",
                ),
                blocked_note: Some("Awaiting security approval for agent deployment."),
            }],
        },
    ]
}
