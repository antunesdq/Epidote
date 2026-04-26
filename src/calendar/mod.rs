use chrono::{Datelike, Duration, NaiveDate};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, FontId, Layout, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui,
};

use crate::{fonts, icons, nav::Open, theme};

const MUTED: Color32 = theme::DIM_TEXT;
const SOFT: Color32 = theme::SOFT_TEXT;
const ERROR: Color32 = theme::ERROR;

// ----- public state -----

pub struct State {
    /// Currently displayed month — only the year/month components matter.
    reference: NaiveDate,
    events: Vec<CalEvent>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            reference: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            events: sample_events(),
        }
    }
}

// ----- model -----

#[derive(Clone, Copy, PartialEq)]
enum CalKind {
    Meeting,
    Deadline,
}

struct CalEvent {
    date: NaiveDate,
    time: Option<&'static str>,
    title: &'static str,
    kind: CalKind,
    target: Option<Open>,
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let mut nav: Option<Open> = None;
    egui::ScrollArea::vertical()
        .id_salt("calendar_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            show_header(ui, state);
            ui.add_space(20.0);
            if let Some(o) = show_grid(ui, state) {
                nav = Some(o);
            }
        });
    nav
}

fn show_header(ui: &mut Ui, state: &mut State) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 44.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                RichText::new(state.reference.format("%B %Y").to_string())
                    .font(fonts::display(28.0))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(8.0);
            if chevron_btn(ui, icons::CHEVRON_LEFT) {
                state.reference = add_months(state.reference, -1);
            }
            if chevron_btn(ui, icons::CHEVRON_RIGHT) {
                state.reference = add_months(state.reference, 1);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("Read-only projection · meetings + task deadlines")
                        .font(FontId::monospace(11.0))
                        .color(SOFT),
                );
            });
        },
    );
}

fn chevron_btn(ui: &mut Ui, glyph: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 3.0, theme::SURFACE_CONTAINER);
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        fonts::icon(14.0),
        if response.hovered() {
            theme::TEXT
        } else {
            SOFT
        },
    );
    response.clicked()
}

// ----- month grid -----

fn show_grid(ui: &mut Ui, state: &State) -> Option<Open> {
    let mut nav: Option<Open> = None;

    let (year, month) = (state.reference.year(), state.reference.month());
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    // Monday-first offset.
    let weekday_idx = first.weekday().num_days_from_monday() as i64;
    let grid_start = first - Duration::days(weekday_idx);
    let days_in_month = last_day_of_month(year, month);
    let total_cells = ((weekday_idx + days_in_month as i64 + 6) / 7 * 7) as usize;
    let rows = total_cells / 7;

    let total_w = ui.available_width();
    let col_w = total_w / 7.0;
    let head_h = 26.0;
    let cell_h = 110.0_f32.min(((ui.available_height() - head_h - 8.0) / rows as f32).max(94.0));

    // Day-name header
    let (head_rect, _) =
        ui.allocate_exact_size(vec2(total_w, head_h), Sense::hover());
    let hp = ui.painter_at(head_rect);
    for (i, label) in ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"]
        .iter()
        .enumerate()
    {
        hp.text(
            pos2(head_rect.left() + i as f32 * col_w + 10.0, head_rect.center().y),
            Align2::LEFT_CENTER,
            *label,
            FontId::monospace(9.5),
            MUTED,
        );
    }

    let today = chrono::Local::now().date_naive();

    // Cells
    let (grid_rect, _) =
        ui.allocate_exact_size(vec2(total_w, cell_h * rows as f32), Sense::hover());
    let painter = ui.painter_at(grid_rect);

    for i in 0..total_cells {
        let row = i / 7;
        let col = i % 7;
        let date = grid_start + Duration::days(i as i64);
        let cell_rect = Rect::from_min_size(
            pos2(grid_rect.left() + col as f32 * col_w, grid_rect.top() + row as f32 * cell_h),
            vec2(col_w, cell_h),
        );

        let in_month = date.month() == month && date.year() == year;
        let is_today = date == today;

        // Cell background + grid lines.
        let bg = if !in_month {
            theme::BACKGROUND
        } else if is_today {
            with_alpha(theme::PRIMARY, 14)
        } else {
            theme::SURFACE_CONTAINER_LOW
        };
        painter.rect_filled(cell_rect, 0.0, bg);
        painter.line_segment(
            [cell_rect.right_top(), cell_rect.right_bottom()],
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        );
        painter.line_segment(
            [cell_rect.left_bottom(), cell_rect.right_bottom()],
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        );
        if is_today {
            painter.rect_stroke(
                cell_rect.shrink(0.5),
                0.0,
                Stroke::new(1.0, with_alpha(theme::PRIMARY, 140)),
                StrokeKind::Inside,
            );
        }

        // Day number
        let num_color = if !in_month {
            MUTED
        } else if is_today {
            theme::PRIMARY
        } else {
            theme::TEXT
        };
        painter.text(
            pos2(cell_rect.left() + 8.0, cell_rect.top() + 8.0),
            Align2::LEFT_TOP,
            date.day().to_string(),
            fonts::display(13.0),
            num_color,
        );
        if is_today {
            painter.circle_filled(
                pos2(cell_rect.right() - 10.0, cell_rect.top() + 13.0),
                3.0,
                theme::PRIMARY,
            );
        }

        // Events stacked under the day number
        let mut y = cell_rect.top() + 28.0;
        let max_events = ((cell_rect.height() - 32.0) / 18.0).floor() as usize;
        let day_events: Vec<&CalEvent> =
            state.events.iter().filter(|e| e.date == date).collect();
        let shown = day_events.len().min(max_events.saturating_sub(1).max(1));

        for event in day_events.iter().take(shown) {
            let ev_rect = Rect::from_min_size(
                pos2(cell_rect.left() + 4.0, y),
                vec2(cell_rect.width() - 8.0, 16.0),
            );
            if event_chip(ui, ev_rect, event) {
                nav = event.target.clone();
            }
            y += 18.0;
        }
        if day_events.len() > shown {
            painter.text(
                pos2(cell_rect.left() + 8.0, y + 1.0),
                Align2::LEFT_TOP,
                format!("+{} more", day_events.len() - shown),
                FontId::proportional(10.0),
                MUTED,
            );
        }
    }

    nav
}

fn event_chip(ui: &mut Ui, rect: Rect, event: &CalEvent) -> bool {
    let response = ui.interact(
        rect,
        egui::Id::new(("cal_event", event.title, event.date.num_days_from_ce())),
        Sense::click(),
    );
    let painter = ui.painter_at(rect);
    let (accent, bg) = match event.kind {
        CalKind::Meeting => {
            let a = theme::PRIMARY;
            (a, with_alpha(a, if response.hovered() { 36 } else { 22 }))
        }
        CalKind::Deadline => {
            let a = ERROR;
            (a, with_alpha(a, if response.hovered() { 40 } else { 24 }))
        }
    };
    painter.rect(
        rect,
        2.0,
        bg,
        Stroke::new(1.0, with_alpha(accent, 90)),
        StrokeKind::Inside,
    );
    // Time prefix (if any) in muted color.
    let mut x = rect.left() + 6.0;
    let font = FontId::proportional(10.5);
    if let Some(t) = event.time {
        let tg = painter.layout_no_wrap(t.to_string(), FontId::monospace(9.5), MUTED);
        painter.text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            t,
            FontId::monospace(9.5),
            MUTED,
        );
        x += tg.size().x + 6.0;
    }
    // Truncate title to fit
    let max_chars = ((rect.width() - (x - rect.left()) - 6.0) / 6.0).floor() as usize;
    let title = if event.title.chars().count() <= max_chars {
        event.title.to_string()
    } else {
        let mut s: String = event.title.chars().take(max_chars.saturating_sub(1)).collect();
        s.push('…');
        s
    };
    painter.text(
        pos2(x, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        font,
        accent,
    );
    response.clicked()
}

// ----- helpers -----

fn add_months(date: NaiveDate, delta: i32) -> NaiveDate {
    let total = date.year() * 12 + (date.month() as i32 - 1) + delta;
    let year = total.div_euclid(12);
    let month = total.rem_euclid(12) as u32 + 1;
    let day = date.day().min(last_day_of_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    first_next
        .map(|d| (d - Duration::days(1)).day())
        .unwrap_or(28)
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// ----- sample data -----

fn sample_events() -> Vec<CalEvent> {
    let d = |y: i32, m: u32, day: u32| NaiveDate::from_ymd_opt(y, m, day).unwrap();
    vec![
        CalEvent {
            date: d(2026, 4, 22),
            time: Some("15:00"),
            title: "System Architecture Sync",
            kind: CalKind::Meeting,
            target: Some(Open::Meeting("System Architecture Sync".into())),
        },
        CalEvent {
            date: d(2026, 4, 23),
            time: Some("10:00"),
            title: "TSK-088 due",
            kind: CalKind::Deadline,
            target: Some(Open::Task("TSK-088".into())),
        },
        CalEvent {
            date: d(2026, 4, 24),
            time: Some("14:00"),
            title: "Q4 Kickoff",
            kind: CalKind::Meeting,
            target: None,
        },
        CalEvent {
            date: d(2026, 4, 24),
            time: Some("17:00"),
            title: "TSK-118 due",
            kind: CalKind::Deadline,
            target: None,
        },
        CalEvent {
            date: d(2026, 4, 26),
            time: None,
            title: "TSK-092 due",
            kind: CalKind::Deadline,
            target: Some(Open::Task("TSK-092".into())),
        },
        CalEvent {
            date: d(2026, 4, 28),
            time: Some("11:00"),
            title: "Security Review",
            kind: CalKind::Meeting,
            target: None,
        },
        CalEvent {
            date: d(2026, 5, 1),
            time: None,
            title: "TSK-104 due",
            kind: CalKind::Deadline,
            target: Some(Open::Task("TSK-104".into())),
        },
        CalEvent {
            date: d(2026, 5, 6),
            time: None,
            title: "TSK-112 due",
            kind: CalKind::Deadline,
            target: Some(Open::Task("TSK-112".into())),
        },
    ]
}
