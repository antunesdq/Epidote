use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2,
};

use crate::{fonts, icons, nav::Open, theme};

const SOFT: Color32 = theme::SOFT_TEXT;
const MUTED: Color32 = theme::DIM_TEXT;
const ERROR: Color32 = theme::ERROR;
const PURPLE: Color32 = theme::ACCENT_PURPLE;
const AMBER: Color32 = theme::ACCENT_AMBER;

const TWO_COL_BREAKPOINT: f32 = 920.0;

pub fn show(ui: &mut Ui) -> Option<Open> {
    let mut nav: Option<Open> = None;
    egui::ScrollArea::vertical()
        .id_salt("today_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_w = ui.available_width();
            let two_col = total_w >= TWO_COL_BREAKPOINT;

            if two_col {
                let gap = 20.0;
                let left_w = ((total_w - gap) * 2.0 / 3.0).floor();
                let right_w = total_w - gap - left_w;

                ui.horizontal_top(|ui| {
                    let origin = ui.cursor().min;
                    let height = ui.available_height().max(560.0);

                    ui.allocate_rect(
                        Rect::from_min_size(origin, vec2(total_w, height)),
                        Sense::hover(),
                    );

                    let left_rect = Rect::from_min_size(origin, vec2(left_w, height));
                    let mut left = ui.new_child(
                        UiBuilder::new()
                            .max_rect(left_rect)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    if let Some(o) = show_left(&mut left) {
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
                    if let Some(o) = show_right(&mut right) {
                        nav = Some(o);
                    }
                });
            } else {
                if let Some(o) = show_left(ui) {
                    nav = Some(o);
                }
                ui.add_space(20.0);
                if let Some(o) = show_right(ui) {
                    nav = Some(o);
                }
            }
        });
    nav
}

// ----- left column -----

fn show_left(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    if let Some(o) = today_hero(ui) {
        nav = Some(o);
    }
    ui.add_space(20.0);
    if let Some(o) = proposals_card(ui) {
        nav = Some(o);
    }
    nav
}

fn today_hero(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    card_frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            day_strip(ui);
            ui.add_space(8.0);

            // Live meeting agenda item.
            if let Some(o) = agenda_item(
                ui,
                AgendaKind::Live,
                "01:24",
                "System Architecture Sync",
                "Project Alpha · Engineering · 5 participants",
                Some(Open::Meeting("System Architecture Sync".to_string())),
            ) {
                nav = Some(o);
            }
            // Scheduled later today.
            if let Some(o) = agenda_item(
                ui,
                AgendaKind::Meeting,
                "17:00",
                "Q4 Kickoff",
                "Scheduled",
                None,
            ) {
                nav = Some(o);
            }
            // Deadline.
            if let Some(o) = agenda_item(
                ui,
                AgendaKind::Deadline,
                "—",
                "TSK-088 due",
                "Task deadline",
                Some(Open::Task("TSK-088".to_string())),
            ) {
                nav = Some(o);
            }
        });
    nav
}

fn day_strip(ui: &mut Ui) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let bullet = "  ·  ";
    let mono = FontId::monospace(12.0);
    let display = fonts::display(13.0);

    let today = chrono::Local::now().date_naive();
    let day_label = today.format("%A").to_string();
    let date_label = today.format(", %b %-d").to_string();

    let mut x = rect.left() + 2.0;
    let y = rect.center().y - 1.0;

    let day_galley =
        painter.layout_no_wrap(day_label.clone(), display.clone(), theme::PRIMARY);
    painter.text(
        pos2(x, y),
        Align2::LEFT_CENTER,
        &day_label,
        display.clone(),
        theme::PRIMARY,
    );
    x += day_galley.size().x;

    let date_galley = painter.layout_no_wrap(date_label.clone(), mono.clone(), SOFT);
    painter.text(
        pos2(x, y),
        Align2::LEFT_CENTER,
        &date_label,
        mono.clone(),
        SOFT,
    );
    x += date_galley.size().x;

    for chunk in [bullet, "3 meetings today", bullet, "2 proposals awaiting review"] {
        let g = painter.layout_no_wrap(chunk.to_string(), mono.clone(), MUTED);
        painter.text(pos2(x, y), Align2::LEFT_CENTER, chunk, mono.clone(), MUTED);
        x += g.size().x;
    }
}

#[derive(Copy, Clone)]
enum AgendaKind {
    Live,
    Meeting,
    Deadline,
}

fn agenda_item(
    ui: &mut Ui,
    kind: AgendaKind,
    time: &str,
    title: &str,
    sub: &str,
    target: Option<Open>,
) -> Option<Open> {
    let h = 56.0;
    let sense = if target.is_some() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), sense);
    let painter = ui.painter_at(rect);
    let hovered = response.hovered() && target.is_some();

    let bg = if hovered {
        theme::SURFACE_CONTAINER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 4.0, bg);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Time label
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        time,
        FontId::monospace(11.5),
        SOFT,
    );

    // State chip / dot
    let chip_x = rect.left() + 70.0;
    match kind {
        AgendaKind::Live => {
            let pill_w = 50.0;
            let pill_rect = Rect::from_min_size(
                pos2(chip_x, rect.center().y - 9.0),
                vec2(pill_w, 18.0),
            );
            painter.rect(
                pill_rect,
                10.0,
                with_alpha(theme::PRIMARY, 30),
                Stroke::new(1.0, with_alpha(theme::PRIMARY, 110)),
                StrokeKind::Inside,
            );
            painter.circle_filled(
                pos2(pill_rect.left() + 10.0, pill_rect.center().y),
                3.0,
                theme::PRIMARY,
            );
            painter.text(
                pos2(pill_rect.left() + 18.0, pill_rect.center().y),
                Align2::LEFT_CENTER,
                "LIVE",
                FontId::monospace(9.5),
                theme::PRIMARY,
            );
        }
        AgendaKind::Meeting => {
            painter.circle_filled(
                pos2(chip_x + 10.0, rect.center().y),
                4.0,
                theme::PRIMARY,
            );
        }
        AgendaKind::Deadline => {
            painter.circle_filled(
                pos2(chip_x + 10.0, rect.center().y),
                4.0,
                ERROR,
            );
        }
    };

    // Title + sub
    let title_x = chip_x + 56.0;
    painter.text(
        pos2(title_x, rect.top() + 12.0),
        Align2::LEFT_TOP,
        title,
        fonts::display(13.5),
        theme::TEXT,
    );
    painter.text(
        pos2(title_x, rect.top() + 30.0),
        Align2::LEFT_TOP,
        sub,
        FontId::proportional(11.0),
        SOFT,
    );

    if hovered {
        painter.text(
            pos2(rect.right() - 12.0, rect.center().y),
            Align2::RIGHT_CENTER,
            icons::ARROW_FORWARD,
            fonts::icon(13.0),
            theme::TEXT,
        );
    }

    if response.clicked() {
        return target;
    }
    None
}

fn proposals_card(ui: &mut Ui) -> Option<Open> {
    let mut nav: Option<Open> = None;
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        card_title(ui, "PROPOSALS AWAITING YOU", Some("4 PENDING"));
        ui.add_space(8.0);

        // (id, kind, title, rationale, source, confidence)
        let proposals = [
            (
                "prop-chunking",
                "Solution",
                "Use 512-token windows with 64-token overlap as the RAG default",
                "Recall@10 peaks at 512/64 on the meetings corpus. Token budget stays within 18% of the 256 baseline.",
                "from System Architecture Sync",
                82,
            ),
            (
                "prop-extract",
                "Task",
                "Create TSK: Benchmark DataLoader batching on GraphQL resolver",
                "Sarah mentioned staging on AWS-East-1 @ 01:23:48. No task captures this; tagging David as owner.",
                "from System Architecture Sync",
                91,
            ),
            (
                "prop-doc-migration",
                "Doc",
                "Generated: Migration Manifesto v1.md",
                "Full architectural proposal extracted from the meeting transcript. 4 sections, references 2 vault notes.",
                "from Q3 Data Architecture",
                79,
            ),
        ];
        for (i, (id, kind, title, rationale, source, conf)) in proposals.iter().enumerate() {
            if proposal_card_compact(ui, kind, title, rationale, source, *conf) {
                nav = Some(Open::Proposal((*id).to_string()));
            }
            if i + 1 < proposals.len() {
                ui.add_space(8.0);
            }
        }
    });
    nav
}

fn proposal_card_compact(
    ui: &mut Ui,
    kind: &str,
    title: &str,
    rationale: &str,
    source: &str,
    conf: u8,
) -> bool {
    let response = egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(*&kind)
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(AMBER),
                );
                ui.label(
                    RichText::new(source)
                        .size(10.5)
                        .color(MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{conf}% confidence"))
                            .font(FontId::monospace(10.0))
                            .color(SOFT),
                    );
                });
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(title)
                    .font(fonts::display(13.5))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(2.0);
            ui.label(RichText::new(rationale).size(12.0).color(SOFT));
        })
        .response;
    let click = ui.interact(
        response.rect,
        response.id.with("today_prop_click"),
        Sense::click(),
    );
    click.clicked()
}

// ----- right column -----

fn show_right(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    if let Some(o) = your_tasks_card(ui) {
        nav = Some(o);
    }
    ui.add_space(20.0);
    if let Some(o) = due_soon_card(ui) {
        nav = Some(o);
    }
    ui.add_space(20.0);
    if let Some(o) = activity_card(ui) {
        nav = Some(o);
    }
    nav
}

fn your_tasks_card(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("YOUR TASKS")
                    .font(FontId::monospace(10.0))
                    .strong()
                    .color(MUTED),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if jump_link(ui, "VIEW BOARD →") {
                    nav = Some(Open::Tab(crate::menu::Nav::Tasks));
                }
            });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            stat(ui, "2", "Active", true);
            stat(ui, "3", "Due soon", false);
            stat(ui, "1", "Blocked", false);
        });
    });
    nav
}

fn jump_link(ui: &mut Ui, label: &str) -> bool {
    let font = FontId::monospace(10.0);
    let g = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), MUTED);
    let w = g.size().x + 12.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 22.0), Sense::click());
    let painter = ui.painter_at(rect);
    let color = if response.hovered() {
        theme::PRIMARY
    } else {
        MUTED
    };
    if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, color);
    response.clicked()
}

fn stat(ui: &mut Ui, num: &str, label: &str, accent: bool) {
    let w = (ui.available_width() / 3.0).max(80.0) - 4.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, 64.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        pos2(rect.left() + 2.0, rect.top() + 4.0),
        Align2::LEFT_TOP,
        num,
        fonts::display(28.0),
        if accent { theme::PRIMARY } else { theme::TEXT },
    );
    painter.text(
        pos2(rect.left() + 2.0, rect.bottom() - 14.0),
        Align2::LEFT_BOTTOM,
        label,
        FontId::proportional(11.0),
        SOFT,
    );
}

fn due_soon_card(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        card_title(ui, "DUE SOON", None);
        ui.add_space(6.0);
        let rows = [
            ("blocked", "TSK-088", "Migrate legacy logging to Datadog cluster", "tomorrow", true),
            ("active", "TSK-118", "Publish Q3 roadmap one-pager", "in 2 days", true),
            ("active", "TSK-092", "Determine optimal chunking strategy for vector embeddings", "in 4 days", true),
        ];
        for (status, id, title, due, soon) in rows.iter() {
            if mini_row(ui, status, id, title, due, *soon) {
                nav = Some(Open::Task(id.to_string()));
            }
        }
    });
    nav
}

fn mini_row(ui: &mut Ui, status: &str, id: &str, title: &str, due: &str, soon: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 28.0),
        Sense::click(),
    );
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }
    let dot_color = match status {
        "blocked" => ERROR,
        "active" => theme::PRIMARY,
        _ => theme::OUTLINE,
    };
    painter.circle_filled(pos2(rect.left() + 8.0, rect.center().y), 3.5, dot_color);
    painter.text(
        pos2(rect.left() + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        id,
        FontId::monospace(10.5),
        SOFT,
    );
    // Truncate title to fit.
    let max_title_chars = 36usize;
    let truncated: String = if title.chars().count() <= max_title_chars {
        title.to_string()
    } else {
        let mut s: String = title.chars().take(max_title_chars - 1).collect();
        s.push('…');
        s
    };
    painter.text(
        pos2(rect.left() + 80.0, rect.center().y),
        Align2::LEFT_CENTER,
        &truncated,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 6.0, rect.center().y),
        Align2::RIGHT_CENTER,
        due,
        FontId::monospace(10.0),
        if soon { theme::PRIMARY } else { MUTED },
    );
    response.clicked()
}

fn activity_card(ui: &mut Ui) -> Option<Open> {
    let mut nav = None;
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        card_title(ui, "ACTIVITY", None);
        ui.add_space(6.0);
        let rows = [
            ("proposal", "AI extracted 2 proposals from System Architecture Sync (live).", "2 min ago", None),
            ("task", "TSK-088 blocked — awaiting security approval on Datadog agent.", "38 min ago", Some(Open::Task("TSK-088".to_string()))),
            ("meeting", "Q3 Roadmap Planning finished processing. Summary ready to review.", "6h ago", Some(Open::Meeting("Q3 Roadmap Planning".to_string()))),
            ("doc", "Migration Manifesto v1 auto-drafted to Vault/Architecture/.", "yesterday", None),
            ("proposal", "Caching RFC ownership proposal generated (confidence 68%).", "2d ago", None),
        ];
        for (kind, text, when, target) in rows {
            if let Some(t) = activity_row(ui, kind, text, when, target.is_some()) {
                if t {
                    nav = target;
                }
            }
        }
    });
    nav
}

fn activity_row(
    ui: &mut Ui,
    kind: &str,
    text: &str,
    when: &str,
    has_target: bool,
) -> Option<bool> {
    // Pre-layout the wrapped text so the row height matches its actual content
    // (otherwise multi-line entries collide with the timestamp below).
    let avail_w = ui.available_width();
    let text_font = FontId::proportional(11.5);
    let text_indent = 22.0;
    let right_pad = 12.0;
    let max_text_w = (avail_w - text_indent - right_pad).max(80.0);
    let galley = ui
        .painter()
        .layout(text.to_string(), text_font.clone(), theme::TEXT, max_text_w);
    let top_pad = 8.0;
    let bottom_pad = 8.0;
    let timestamp_h = 14.0;
    let h = top_pad + galley.size().y + 4.0 + timestamp_h + bottom_pad;

    let sense = if has_target {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(avail_w, h), sense);
    let painter = ui.painter_at(rect);
    let hovered = response.hovered() && has_target;
    if hovered {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    let dot_color = match kind {
        "proposal" => AMBER,
        "task" => ERROR,
        "meeting" => PURPLE,
        "doc" => theme::PRIMARY,
        _ => theme::OUTLINE,
    };
    painter.circle_filled(
        pos2(rect.left() + 8.0, rect.top() + top_pad + 7.0),
        3.5,
        dot_color,
    );

    painter.galley(
        pos2(rect.left() + text_indent, rect.top() + top_pad),
        galley,
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + text_indent, rect.bottom() - bottom_pad),
        Align2::LEFT_BOTTOM,
        when,
        FontId::monospace(9.5),
        MUTED,
    );

    Some(response.clicked())
}

// ----- helpers -----

fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(theme::SURFACE_CONTAINER_LOW)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .inner_margin(Margin::same(18))
}

fn card_title(ui: &mut Ui, label: &str, trailing: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::monospace(10.0))
                .strong()
                .color(MUTED),
        );
        if let Some(t) = trailing {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(t)
                        .font(FontId::monospace(10.0))
                        .color(MUTED),
                );
            });
        }
    });
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn _vec(_: Vec2) {}
