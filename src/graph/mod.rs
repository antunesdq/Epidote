use std::path::PathBuf;

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Id, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2,
};

use crate::{fonts, icons, nav::Open, theme};

// ----- public state -----

pub struct State {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    selected: Option<usize>,
    show_meetings: bool,
    show_docs: bool,
    show_tasks: bool,
    pan: Vec2,
    zoom: f32,
}

impl Default for State {
    fn default() -> Self {
        let (nodes, edges) = sample_graph();
        Self {
            nodes,
            edges,
            selected: None,
            show_meetings: true,
            show_docs: true,
            show_tasks: true,
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

// ----- model -----

#[derive(Clone, Copy, PartialEq)]
enum NodeKind {
    Meeting,
    Document,
    Task,
}

impl NodeKind {
    fn color(self) -> Color32 {
        match self {
            NodeKind::Meeting => theme::PRIMARY,
            NodeKind::Document => Color32::from_rgb(0xce, 0xbd, 0xff),
            NodeKind::Task => Color32::from_rgb(0xff, 0xb4, 0xab),
        }
    }
    fn glyph(self) -> &'static str {
        match self {
            NodeKind::Meeting => icons::VIDEO_LIBRARY,
            NodeKind::Document => icons::DESCRIPTION,
            NodeKind::Task => icons::TASK_ALT,
        }
    }
    fn label(self) -> &'static str {
        match self {
            NodeKind::Meeting => "Meeting",
            NodeKind::Document => "Document",
            NodeKind::Task => "Task",
        }
    }
}

struct Node {
    kind: NodeKind,
    label: String,
    pos: Pos2,
    radius: f32,
    summary: String,
    target: Option<Target>,
}

#[derive(Clone)]
enum Target {
    Vault(PathBuf),
    Task(String),
}

impl Target {
    fn as_open(&self) -> Open {
        match self {
            Target::Vault(p) => Open::Vault(p.clone()),
            Target::Task(id) => Open::Task(id.clone()),
        }
    }

    fn cta(&self) -> &'static str {
        match self {
            Target::Vault(_) => "OPEN IN VAULT",
            Target::Task(_) => "OPEN TASK",
        }
    }
}

struct Edge {
    a: usize,
    b: usize,
    strength: EdgeStrength,
}

#[derive(Clone, Copy)]
enum EdgeStrength {
    Strong,
    Normal,
    Weak,
}

impl EdgeStrength {
    fn label(self) -> &'static str {
        match self {
            EdgeStrength::Strong => "Strong Link",
            EdgeStrength::Normal => "Linked",
            EdgeStrength::Weak => "Weak Link",
        }
    }
    fn alpha(self) -> u8 {
        match self {
            EdgeStrength::Strong => 140,
            EdgeStrength::Normal => 90,
            EdgeStrength::Weak => 55,
        }
    }
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let total = ui.available_size_before_wrap();
    let origin = ui.cursor().min;
    let sidebar_w = 220.0;
    let gap = 12.0;

    let sidebar_rect = Rect::from_min_size(origin, vec2(sidebar_w, total.y));
    let canvas_rect = Rect::from_min_size(
        pos2(origin.x + sidebar_w + gap, origin.y),
        vec2(total.x - sidebar_w - gap, total.y),
    );

    ui.allocate_rect(
        Rect::from_min_size(origin, vec2(total.x, total.y)),
        Sense::hover(),
    );

    let mut sidebar_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(sidebar_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    show_filters(&mut sidebar_ui, state);

    show_canvas(ui, canvas_rect, state);
    show_view_controls(ui, canvas_rect, state);

    let mut nav: Option<Open> = None;
    if let Some(idx) = state.selected {
        if idx < state.nodes.len() {
            nav = show_inspector(ui, canvas_rect, state, idx);
        }
    }
    nav
}

// ----- filter sidebar -----

fn show_filters(ui: &mut Ui, state: &mut State) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 4.0, theme::SURFACE_CONTAINER_LOW);

    let inner = outer.shrink(14.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    child.label(
        RichText::new("GRAPH FILTERS")
            .font(fonts::display(11.0))
            .strong()
            .color(theme::DIM_TEXT),
    );
    child.add_space(12.0);

    filter_row(&mut child, &mut state.show_meetings, "Meetings", NodeKind::Meeting);
    child.add_space(6.0);
    filter_row(
        &mut child,
        &mut state.show_docs,
        "Documentation",
        NodeKind::Document,
    );
    child.add_space(6.0);
    filter_row(&mut child, &mut state.show_tasks, "Tasks", NodeKind::Task);

    child.add_space(16.0);
    let sep_rect = child.cursor().min;
    let sep_w = child.available_width();
    child.painter().line_segment(
        [sep_rect, pos2(sep_rect.x + sep_w, sep_rect.y)],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );
    child.add_space(12.0);

    child.label(
        RichText::new(format!("Nodes: {}", state.nodes.len()))
            .size(11.0)
            .color(theme::DIM_TEXT),
    );
    child.label(
        RichText::new(format!("Edges: {}", state.edges.len()))
            .size(11.0)
            .color(theme::DIM_TEXT),
    );
}

fn filter_row(ui: &mut Ui, value: &mut bool, label: &str, kind: NodeKind) {
    let row_h = 24.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }

    // Checkbox square
    let cb_rect = Rect::from_min_size(pos2(rect.left() + 2.0, rect.center().y - 7.0), vec2(14.0, 14.0));
    painter.rect(
        cb_rect,
        2.0,
        if *value {
            theme::PRIMARY
        } else {
            theme::SURFACE_CONTAINER
        },
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    if *value {
        painter.text(
            cb_rect.center(),
            Align2::CENTER_CENTER,
            "\u{e5ca}", // check
            fonts::icon(11.0),
            theme::BUTTON_TEXT,
        );
    }

    // Color dot
    painter.circle_filled(
        pos2(rect.left() + 28.0, rect.center().y),
        4.0,
        kind.color(),
    );
    // Label
    painter.text(
        pos2(rect.left() + 42.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        theme::TEXT,
    );

    if response.clicked() {
        *value = !*value;
    }
}

fn is_visible(kind: NodeKind, state: &State) -> bool {
    match kind {
        NodeKind::Meeting => state.show_meetings,
        NodeKind::Document => state.show_docs,
        NodeKind::Task => state.show_tasks,
    }
}

// ----- canvas -----

fn show_canvas(ui: &mut Ui, rect: Rect, state: &mut State) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, theme::SURFACE_CONTAINER_LOW);

    // Subtle radial-like vignette via concentric faint rings.
    let center = rect.center();
    for i in 1..=4 {
        let r = i as f32 * 80.0;
        painter.circle_stroke(
            center,
            r,
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(0, 255, 136, (8 - i as u8 * 2).max(2)),
            ),
        );
    }

    let response = ui.interact(rect, Id::new("graph_canvas"), Sense::click_and_drag());

    if response.dragged() {
        state.pan += response.drag_delta();
    }
    if response.hovered() {
        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.1 {
            let factor = (scroll * 0.0025).exp();
            state.zoom = (state.zoom * factor).clamp(0.4, 2.5);
        }
    }

    let zoom = state.zoom;
    let pan = state.pan;
    let to_screen = |p: Pos2| -> Pos2 {
        pos2(
            center.x + (p.x - 0.0) * zoom + pan.x,
            center.y + (p.y - 0.0) * zoom + pan.y,
        )
    };

    // Draw edges
    for edge in &state.edges {
        let na = &state.nodes[edge.a];
        let nb = &state.nodes[edge.b];
        if !is_visible(na.kind, state) || !is_visible(nb.kind, state) {
            continue;
        }
        let pa = to_screen(na.pos);
        let pb = to_screen(nb.pos);
        let mid = blend(na.kind.color(), nb.kind.color());
        let color = with_alpha(mid, edge.strength.alpha());
        painter.line_segment([pa, pb], Stroke::new(1.5, color));
    }

    // Draw nodes
    let pointer = ui.ctx().pointer_hover_pos();
    let mut hover_idx: Option<usize> = None;

    for (i, node) in state.nodes.iter().enumerate() {
        if !is_visible(node.kind, state) {
            continue;
        }
        let p = to_screen(node.pos);
        let r = node.radius * zoom;
        let color = node.kind.color();

        let hot = pointer
            .map(|m| (m - p).length() <= r && rect.contains(m))
            .unwrap_or(false);
        if hot {
            hover_idx = Some(i);
        }
        let is_selected = state.selected == Some(i);

        // Background disc
        painter.circle_filled(p, r, theme::SURFACE_HIGH);
        // Outer glow when selected
        if is_selected {
            painter.circle_stroke(p, r + 5.0, Stroke::new(1.0, with_alpha(color, 60)));
            painter.circle_stroke(p, r + 9.0, Stroke::new(1.0, with_alpha(color, 30)));
        }
        // Border
        let stroke = if is_selected || hot {
            Stroke::new(2.0, color)
        } else {
            Stroke::new(1.0, with_alpha(color, 110))
        };
        painter.circle_stroke(p, r, stroke);
        // Icon
        painter.text(
            p,
            Align2::CENTER_CENTER,
            node.kind.glyph(),
            fonts::icon((r * 0.95).max(10.0)),
            color,
        );

        // Label
        if hot || is_selected || r >= 22.0 {
            painter.text(
                pos2(p.x, p.y + r + 8.0),
                Align2::CENTER_TOP,
                &node.label,
                FontId::proportional(11.0),
                theme::TEXT,
            );
        }
    }

    if response.clicked() {
        state.selected = hover_idx;
    }
}

fn blend(a: Color32, b: Color32) -> Color32 {
    Color32::from_rgb(
        ((a.r() as u16 + b.r() as u16) / 2) as u8,
        ((a.g() as u16 + b.g() as u16) / 2) as u8,
        ((a.b() as u16 + b.b() as u16) / 2) as u8,
    )
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// ----- bottom-left view controls -----

fn show_view_controls(ui: &mut Ui, canvas_rect: Rect, state: &mut State) {
    let pad = 12.0;
    let h = 36.0;
    let group_w = 110.0;
    let origin = pos2(canvas_rect.left() + pad, canvas_rect.bottom() - h - pad);

    // Zoom group: in / out / recenter
    let group_rect = Rect::from_min_size(origin, vec2(group_w, h));
    ui.painter().rect(
        group_rect,
        4.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );

    let btn_w = group_w / 3.0;
    let make_btn = |i: usize, glyph: &str| -> (Rect, egui::Response) {
        let r = Rect::from_min_size(
            pos2(origin.x + i as f32 * btn_w, origin.y),
            vec2(btn_w, h),
        );
        let resp = ui.interact(r, Id::new(("graph_ctl", glyph)), Sense::click());
        let p = ui.painter_at(r);
        if resp.hovered() {
            p.rect_filled(r, 0.0, theme::SURFACE_HIGH);
        }
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            glyph,
            fonts::icon(16.0),
            if resp.hovered() { theme::TEXT } else { theme::DIM_TEXT },
        );
        (r, resp)
    };

    let (_, zin) = make_btn(0, icons::ZOOM_IN);
    let (_, zout) = make_btn(1, icons::ZOOM_OUT);
    let (_, recenter) = make_btn(2, icons::MY_LOCATION);

    if zin.clicked() {
        state.zoom = (state.zoom * 1.2).clamp(0.4, 2.5);
    }
    if zout.clicked() {
        state.zoom = (state.zoom / 1.2).clamp(0.4, 2.5);
    }
    if recenter.clicked() {
        state.pan = Vec2::ZERO;
        state.zoom = 1.0;
    }
}

// ----- floating inspector -----

fn show_inspector(ui: &mut Ui, canvas_rect: Rect, state: &mut State, idx: usize) -> Option<Open> {
    let panel_w = 300.0;
    let pad = 16.0;
    let origin = pos2(canvas_rect.right() - panel_w - pad, canvas_rect.top() + pad);
    let panel_rect = Rect::from_min_size(
        origin,
        vec2(panel_w, canvas_rect.height() - pad * 2.0),
    );

    ui.painter().rect(
        panel_rect,
        4.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );

    let inner = panel_rect.shrink(16.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    let node = &state.nodes[idx];
    let kind_label = node.kind.label();
    let kind_color = node.kind.color();
    let label = node.label.clone();
    let summary = node.summary.clone();
    let target = node.target.clone();
    let mut open_req: Option<Open> = None;

    // Header row: kind tag + close
    child.horizontal(|ui| {
        ui.label(
            RichText::new(format!("● {}", kind_label.to_uppercase()))
                .size(10.5)
                .strong()
                .color(kind_color),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (rect, response) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                icons::CLOSE,
                fonts::icon(14.0),
                if response.hovered() {
                    theme::TEXT
                } else {
                    theme::DIM_TEXT
                },
            );
            if response.clicked() {
                state.selected = None;
            }
        });
    });
    child.add_space(2.0);
    child.label(
        RichText::new(&label)
            .font(fonts::display(18.0))
            .strong()
            .color(theme::TEXT),
    );

    child.add_space(14.0);
    child.label(
        RichText::new("SUMMARY")
            .font(fonts::display(10.0))
            .strong()
            .color(theme::DIM_TEXT),
    );
    child.add_space(4.0);

    // Summary card — clickable when the node has a navigable target.
    let summary_lines = (summary.len() / 60 + 1).clamp(2, 6) as f32;
    let cta_h = if target.is_some() { 22.0 } else { 0.0 };
    let summary_h = 20.0 + summary_lines * 16.0 + cta_h;
    let sense = if target.is_some() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (sum_rect, sum_response) =
        child.allocate_exact_size(vec2(child.available_width(), summary_h), sense);
    let hovered = sum_response.hovered() && target.is_some();
    let sp = child.painter_at(sum_rect);
    let border = if hovered {
        Stroke::new(1.0, kind_color)
    } else {
        Stroke::new(1.0, theme::OUTLINE_VARIANT)
    };
    let fill = if hovered {
        theme::SURFACE_CONTAINER
    } else {
        theme::SURFACE_CONTAINER_LOW
    };
    sp.rect(sum_rect, 2.0, fill, border, StrokeKind::Inside);

    let text_rect = Rect::from_min_size(
        sum_rect.min + vec2(10.0, 10.0),
        vec2(sum_rect.width() - 20.0, summary_h - 20.0 - cta_h),
    );
    let mut text_ui = child.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    text_ui.label(RichText::new(&summary).size(11.5).color(theme::TEXT));

    if let Some(t) = &target {
        sp.text(
            pos2(sum_rect.left() + 10.0, sum_rect.bottom() - 11.0),
            Align2::LEFT_CENTER,
            t.cta(),
            FontId::proportional(10.0),
            if hovered { kind_color } else { theme::DIM_TEXT },
        );
        sp.text(
            pos2(sum_rect.right() - 10.0, sum_rect.bottom() - 11.0),
            Align2::RIGHT_CENTER,
            icons::OPEN_IN_NEW,
            fonts::icon(12.0),
            if hovered { kind_color } else { theme::DIM_TEXT },
        );
    }

    if sum_response.clicked() {
        if let Some(t) = &target {
            open_req = Some(t.as_open());
        }
    }

    child.add_space(14.0);

    // Connected entities
    let connected: Vec<(usize, EdgeStrength)> = state
        .edges
        .iter()
        .filter_map(|e| {
            if e.a == idx {
                Some((e.b, e.strength))
            } else if e.b == idx {
                Some((e.a, e.strength))
            } else {
                None
            }
        })
        .collect();

    child.horizontal(|ui| {
        ui.label(
            RichText::new("CONNECTED ENTITIES")
                .font(fonts::display(10.0))
                .strong()
                .color(theme::DIM_TEXT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}", connected.len()))
                    .size(10.5)
                    .color(theme::DIM_TEXT),
            );
        });
    });
    child.add_space(6.0);

    let mut goto: Option<usize> = None;
    egui::ScrollArea::vertical()
        .id_salt(("graph_inspector_scroll", idx))
        .auto_shrink([false, false])
        .max_height(child.available_height() - 60.0)
        .show(&mut child, |ui| {
            for (other, strength) in &connected {
                if connection_row(ui, &state.nodes[*other], *strength) {
                    goto = Some(*other);
                }
            }
        });
    if let Some(other) = goto {
        state.selected = Some(other);
    }

    open_req
}

fn connection_row(ui: &mut Ui, node: &Node, strength: EdgeStrength) -> bool {
    let row_h = 38.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect(
            rect,
            2.0,
            theme::SURFACE_CONTAINER_LOW,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
            StrokeKind::Inside,
        );
    }
    // Color disc
    let color = node.kind.color();
    let disc_center = pos2(rect.left() + 16.0, rect.center().y);
    painter.circle_filled(disc_center, 11.0, with_alpha(color, 30));
    painter.text(
        disc_center,
        Align2::CENTER_CENTER,
        node.kind.glyph(),
        fonts::icon(12.0),
        color,
    );
    // Title + meta
    painter.text(
        pos2(rect.left() + 36.0, rect.top() + 6.0),
        Align2::LEFT_TOP,
        &node.label,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 36.0, rect.top() + 22.0),
        Align2::LEFT_TOP,
        format!("{} · {}", node.kind.label(), strength.label()),
        FontId::proportional(10.0),
        theme::DIM_TEXT,
    );
    if response.hovered() {
        painter.text(
            pos2(rect.right() - 12.0, rect.center().y),
            Align2::RIGHT_CENTER,
            icons::ARROW_FORWARD,
            fonts::icon(13.0),
            theme::DIM_TEXT,
        );
    }
    response.clicked()
}

// ----- sample data -----

fn vault_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Documents/Epidote/Vault")
}

fn sample_graph() -> (Vec<Node>, Vec<Edge>) {
    let root = vault_root();
    let doc = |rel: &str| Target::Vault(root.join(rel));
    let task = |id: &str| Target::Task(id.to_string());

    let nodes = vec![
        Node {
            kind: NodeKind::Document,
            label: "Embedding Architecture".to_string(),
            pos: pos2(0.0, 0.0),
            radius: 28.0,
            summary: "RAG pipeline uses 1024-token windows with 128-token overlap. Tracks \
                      open questions around per-section chunking."
                .to_string(),
            target: Some(doc("Architecture/Embedding Architecture.md")),
        },
        Node {
            kind: NodeKind::Document,
            label: "Chunking Benchmarks".to_string(),
            pos: pos2(-220.0, 110.0),
            radius: 22.0,
            summary: "256/512/1024-token windows × 0/64/128 overlap. Recall@10 peaks at 512/64 \
                      on the meetings corpus."
                .to_string(),
            target: Some(doc("Architecture/Chunking Benchmarks.md")),
        },
        Node {
            kind: NodeKind::Document,
            label: "Caching RFC v2".to_string(),
            pos: pos2(-180.0, -140.0),
            radius: 22.0,
            summary: "Write-through cache with TTL bucketing. Targets 92%+ hit rate on hot \
                      keys without blowing the memory ceiling."
                .to_string(),
            target: Some(doc("Architecture/Caching RFC v2.md")),
        },
        Node {
            kind: NodeKind::Meeting,
            label: "Q3 Sync".to_string(),
            pos: pos2(-320.0, -40.0),
            radius: 24.0,
            summary: "Quarterly architecture sync. Surfaced caching bottlenecks and triaged \
                      ownership of the platform layer."
                .to_string(),
            target: Some(doc("Meetings/Q3 Sync.md")),
        },
        Node {
            kind: NodeKind::Meeting,
            label: "RAG Pipeline Optimization".to_string(),
            pos: pos2(110.0, 180.0),
            radius: 24.0,
            summary: "Reviewed retrieval quality on the meetings corpus. Action items captured \
                      under TSK-092."
                .to_string(),
            target: Some(doc("Meetings/RAG Pipeline Optimization.md")),
        },
        Node {
            kind: NodeKind::Task,
            label: "TSK-092 · Vector Chunking".to_string(),
            pos: pos2(240.0, 60.0),
            radius: 18.0,
            summary: "Compare 256/512/1024-token windows with 0/64/128-token overlap on the \
                      meetings corpus. Land on a default before RAG cutover."
                .to_string(),
            target: Some(task("TSK-092")),
        },
        Node {
            kind: NodeKind::Task,
            label: "TSK-104 · Redis Caching".to_string(),
            pos: pos2(-280.0, 260.0),
            radius: 18.0,
            summary: "Cache hit rate on hot keys is plateauing at ~78%. Evaluate the write-\
                      through proposal and call out memory ceiling implications."
                .to_string(),
            target: Some(task("TSK-104")),
        },
        Node {
            kind: NodeKind::Task,
            label: "TSK-088 · Datadog Migration".to_string(),
            pos: pos2(260.0, -160.0),
            radius: 18.0,
            summary: "Switch the legacy syslog forwarder to the Datadog agent in two phases. \
                      Currently blocked on security review."
                .to_string(),
            target: Some(task("TSK-088")),
        },
    ];
    let edges = vec![
        // RAG meeting → docs & tasks it produced
        Edge { a: 4, b: 0, strength: EdgeStrength::Strong },
        Edge { a: 4, b: 1, strength: EdgeStrength::Strong },
        Edge { a: 4, b: 5, strength: EdgeStrength::Strong },
        // Q3 sync → caching work
        Edge { a: 3, b: 2, strength: EdgeStrength::Strong },
        Edge { a: 3, b: 6, strength: EdgeStrength::Strong },
        // Embedding ↔ Chunking Benchmarks
        Edge { a: 0, b: 1, strength: EdgeStrength::Normal },
        // Caching RFC ↔ Redis Caching task
        Edge { a: 2, b: 6, strength: EdgeStrength::Normal },
        // Datadog migration dangles weakly
        Edge { a: 7, b: 3, strength: EdgeStrength::Weak },
    ];
    (nodes, edges)
}
