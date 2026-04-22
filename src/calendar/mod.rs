use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Timelike};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Context, FontId, Id, Layout, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};

use crate::{fonts, icons, theme};

// ----- public state -----

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Month,
    Week,
    Day,
}

pub struct State {
    view: View,
    reference: NaiveDate,
    selected_day: Option<NaiveDate>,
    events: Vec<Event>,
    /// One-shot flag: the first time a time grid renders, we scroll to 07:00
    /// so business hours are visible. After that we honor whatever scroll
    /// position egui has remembered.
    auto_scrolled: bool,
}

impl Default for State {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            view: View::Month,
            reference: today,
            selected_day: None,
            events: sample_events(today),
            auto_scrolled: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum EventKind {
    Meeting,
    Task,
    Deadline,
}

impl EventKind {
    fn color(self) -> Color32 {
        // Mirrors the Tailwind reference: primary-container, secondary, error.
        match self {
            EventKind::Meeting => Color32::from_rgb(0x00, 0xff, 0x88),
            EventKind::Task => Color32::from_rgb(0xce, 0xbd, 0xff),
            EventKind::Deadline => Color32::from_rgb(0xff, 0xb4, 0xab),
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            EventKind::Meeting => icons::VIDEO_LIBRARY,
            EventKind::Task => icons::TASK_ALT,
            EventKind::Deadline => icons::PRIORITY_HIGH,
        }
    }

    fn label(self) -> &'static str {
        match self {
            EventKind::Meeting => "Meeting",
            EventKind::Task => "Task",
            EventKind::Deadline => "Deadline",
        }
    }
}

#[derive(Clone)]
pub struct Event {
    date: NaiveDate,
    kind: EventKind,
    title: String,
    start: NaiveTime,
    duration_minutes: u32,
}

impl Event {
    fn end(&self) -> NaiveTime {
        self.start + Duration::minutes(self.duration_minutes as i64)
    }

    fn time_range(&self) -> String {
        format!(
            "{} – {}",
            self.start.format("%H:%M"),
            self.end().format("%H:%M")
        )
    }
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) {
    show_header(ui, state);
    ui.add_space(12.0);

    let view = state.view;
    let reference = state.reference;
    // Immutable snapshot; `selected_day` and `auto_scrolled` are still borrowed
    // mutably below from disjoint fields of `state`.
    let events = state.events.clone();

    match view {
        View::Month => show_month_grid(ui, reference, &events, &mut state.selected_day),
        View::Week => show_time_grid(
            ui,
            week_start(reference),
            5,
            &events,
            &mut state.selected_day,
            &mut state.auto_scrolled,
        ),
        View::Day => show_time_grid(
            ui,
            reference,
            1,
            &events,
            &mut state.selected_day,
            &mut state.auto_scrolled,
        ),
    }

    ui.add_space(10.0);
    show_legend(ui);

    if let Some(day) = state.selected_day {
        show_day_popup(ui.ctx(), day, &events, &mut state.selected_day);
    }
}

/// Mon–Fri of the ISO week containing `reference`.
fn week_start(reference: NaiveDate) -> NaiveDate {
    reference - Duration::days(reference.weekday().num_days_from_monday() as i64)
}

// ----- header: title + view switcher + prev/next -----

fn show_header(ui: &mut Ui, state: &mut State) {
    // Bound the header to an explicit height. Using bare `with_layout` with
    // `Align::Center` would expand to the parent's full available height,
    // leaving no vertical room for the grid below.
    let header_h = 52.0;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), header_h),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Calendar")
                        .font(fonts::display(28.0))
                        .strong()
                        .color(theme::TEXT),
                );
                ui.add_space(-2.0);
                ui.label(
                    RichText::new(header_subtitle(state))
                        .size(12.5)
                        .color(theme::DIM_TEXT),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Prev/next chevrons — RTL so the rightmost call is added first.
                if icon_button(ui, icons::CHEVRON_RIGHT).clicked() {
                    step_date(&mut state.reference, state.view, 1);
                }
                ui.add_space(4.0);
                if icon_button(ui, icons::CHEVRON_LEFT).clicked() {
                    step_date(&mut state.reference, state.view, -1);
                }

                ui.add_space(12.0);

                // View switcher. Added right-to-left so display order is Month | Week | Day.
                view_pill(ui, &mut state.view, View::Day, "Day");
                view_pill(ui, &mut state.view, View::Week, "Week");
                view_pill(ui, &mut state.view, View::Month, "Month");
            });
        },
    );
}

fn header_subtitle(state: &State) -> String {
    match state.view {
        View::Month => state.reference.format("%B %Y").to_string(),
        View::Week => {
            let start = week_start(state.reference);
            let end = start + Duration::days(4);
            if start.month() == end.month() {
                format!("{} – {} {}", start.format("%b %-d"), end.format("%-d"), start.format("%Y"))
            } else {
                format!(
                    "{} – {} {}",
                    start.format("%b %-d"),
                    end.format("%b %-d"),
                    start.format("%Y")
                )
            }
        }
        View::Day => state.reference.format("%A, %B %-d, %Y").to_string(),
    }
}

fn icon_button(ui: &mut Ui, glyph: &'static str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    let painter = ui.painter_at(rect);
    let fill = if response.hovered() {
        theme::SURFACE_CONTAINER
    } else {
        theme::SURFACE_CONTAINER_LOW
    };
    painter.rect(
        rect,
        2.0,
        fill,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let color = if response.hovered() { theme::PRIMARY } else { theme::TEXT };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        fonts::icon(16.0),
        color,
    );
    response
}

fn view_pill(ui: &mut Ui, current: &mut View, target: View, label: &str) {
    let active = *current == target;
    let (rect, response) = ui.allocate_exact_size(vec2(58.0, 28.0), Sense::click());
    let painter = ui.painter_at(rect);
    let bg = if active {
        theme::SURFACE_CONTAINER
    } else if response.hovered() {
        theme::SURFACE_CONTAINER_LOW
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 2.0, bg);
    let color = if active { theme::TEXT } else { theme::DIM_TEXT };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(11.5),
        color,
    );
    if response.clicked() {
        *current = target;
    }
}

fn step_date(date: &mut NaiveDate, view: View, direction: i32) {
    *date = match view {
        View::Month => add_months(*date, direction),
        View::Week => *date + Duration::days(7 * direction as i64),
        View::Day => *date + Duration::days(direction as i64),
    };
}

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

// ----- month grid (header + body share the same column math) -----

fn show_month_grid(
    ui: &mut Ui,
    reference: NaiveDate,
    events: &[Event],
    selected_day: &mut Option<NaiveDate>,
) {
    let today = chrono::Local::now().date_naive();
    let first_of_month = reference.with_day(1).unwrap_or(reference);
    let offset = first_of_month.weekday().num_days_from_sunday() as i64;
    let grid_start = first_of_month - Duration::days(offset);

    let header_h = 28.0;
    let rows = 6;
    // Reserve ~48px for the legend + breathing room below the grid.
    let body_h = (ui.available_height() - header_h - 48.0).max(88.0 * rows as f32);
    let row_h = body_h / rows as f32;
    let total_h = header_h + body_h;

    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), total_h), Sense::hover());
    let col_w = rect.width() / 7.0;

    // Header strip.
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), header_h));
    let painter = ui.painter();
    painter.rect_filled(header_rect, 0.0, theme::SURFACE_CONTAINER_LOW);
    painter.line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );
    for (i, label) in ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]
        .iter()
        .enumerate()
    {
        painter.text(
            pos2(
                rect.left() + i as f32 * col_w + 10.0,
                header_rect.center().y,
            ),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(10.5),
            theme::DIM_TEXT,
        );
    }

    // Body cells at manual positions — guaranteed to line up with the headers.
    for week in 0..rows {
        for col in 0..7 {
            let cell_rect = Rect::from_min_size(
                pos2(
                    rect.left() + col as f32 * col_w,
                    rect.top() + header_h + week as f32 * row_h,
                ),
                vec2(col_w, row_h),
            );
            let day = grid_start + Duration::days((week * 7 + col) as i64);
            day_cell(
                ui,
                cell_rect,
                day,
                reference.month(),
                today,
                events,
                selected_day,
            );
        }
    }
}

fn day_cell(
    ui: &mut Ui,
    rect: Rect,
    day: NaiveDate,
    reference_month: u32,
    today: NaiveDate,
    events: &[Event],
    selected_day: &mut Option<NaiveDate>,
) {
    let response = ui.interact(rect, Id::new(("cal_month_cell", day)), Sense::click());
    let painter = ui.painter_at(rect);

    let same_month = day.month() == reference_month;
    let is_today = day == today;

    let bg = if !same_month {
        theme::BACKGROUND
    } else if is_today {
        theme::SURFACE_CONTAINER_LOW
    } else if response.hovered() {
        theme::SURFACE_HIGH
    } else {
        theme::SURFACE_CONTAINER
    };
    painter.rect_filled(rect, 0.0, bg);

    // Subtle grid separators mimic the HTML's gap:1px look.
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0, theme::SURFACE_CONTAINER_LOW),
    );
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::SURFACE_CONTAINER_LOW),
    );

    if is_today {
        painter.rect_stroke(
            rect.shrink(0.5),
            0.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 136, 110)),
            StrokeKind::Inside,
        );
    }

    let num_color = if !same_month {
        theme::DIM_TEXT
    } else if is_today {
        theme::PRIMARY
    } else {
        theme::TEXT
    };
    painter.text(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        day.day().to_string(),
        fonts::display(13.0),
        num_color,
    );
    if is_today {
        painter.circle_filled(
            pos2(rect.right() - 10.0, rect.top() + 13.0),
            2.5,
            theme::PRIMARY,
        );
    }

    // Event chips; "+N more" when they don't all fit.
    let day_events: Vec<&Event> = events.iter().filter(|e| e.date == day).collect();
    let marker_h = 16.0;
    let max_markers = ((rect.height() - 34.0) / (marker_h + 2.0)).floor() as usize;
    let shown = day_events
        .len()
        .min(max_markers.saturating_sub(1).max(1));
    let mut y = rect.top() + 26.0;
    for event in day_events.iter().take(shown) {
        draw_event_chip(
            &painter,
            Rect::from_min_size(
                pos2(rect.left() + 4.0, y),
                vec2(rect.width() - 8.0, marker_h),
            ),
            event,
        );
        y += marker_h + 2.0;
    }
    if day_events.len() > shown {
        painter.text(
            pos2(rect.left() + 8.0, y + 1.0),
            Align2::LEFT_TOP,
            format!("+{} more", day_events.len() - shown),
            FontId::proportional(10.0),
            theme::DIM_TEXT,
        );
    }

    if response.clicked() {
        *selected_day = Some(day);
    }
}

fn draw_event_chip(painter: &egui::Painter, rect: Rect, event: &Event) {
    let color = event.kind.color();
    let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 26);
    painter.rect_filled(rect, 2.0, bg);
    // Left accent stripe.
    let stripe = Rect::from_min_size(rect.min, vec2(2.0, rect.height()));
    painter.rect_filled(stripe, 0.0, color);

    painter.text(
        pos2(rect.left() + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        event.kind.glyph(),
        fonts::icon(11.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        &event.title,
        FontId::proportional(10.5),
        color,
    );
}

// ----- time grid (Outlook-style) for week & day -----

const HOUR_H: f32 = 48.0;
const HOUR_COL_W: f32 = 58.0;
const HOURS: u32 = 24;

fn show_time_grid(
    ui: &mut Ui,
    start_date: NaiveDate,
    days: usize,
    events: &[Event],
    selected_day: &mut Option<NaiveDate>,
    auto_scrolled: &mut bool,
) {
    let today = chrono::Local::now().date_naive();
    let total_w = ui.available_width();
    let day_cols_w = total_w - HOUR_COL_W;
    let day_w = day_cols_w / days as f32;

    // -- Pinned day-header row --
    let header_h = 60.0;
    let (header_rect, _) =
        ui.allocate_exact_size(vec2(total_w, header_h), Sense::hover());
    let hp = ui.painter_at(header_rect);
    hp.rect_filled(header_rect, 0.0, theme::SURFACE_CONTAINER_LOW);
    hp.line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Blank corner above the hour column.
    for i in 0..days {
        let day = start_date + Duration::days(i as i64);
        let x = header_rect.left() + HOUR_COL_W + i as f32 * day_w;
        let col_rect = Rect::from_min_size(pos2(x, header_rect.top()), vec2(day_w, header_h));
        let response = ui.interact(
            col_rect,
            Id::new(("cal_time_hdr", day)),
            Sense::click(),
        );

        if response.hovered() {
            hp.rect_filled(col_rect, 0.0, theme::SURFACE_CONTAINER);
        }

        let is_today = day == today;
        hp.text(
            pos2(x + 14.0, header_rect.top() + 10.0),
            Align2::LEFT_TOP,
            day.format("%a").to_string().to_uppercase(),
            FontId::proportional(10.5),
            theme::DIM_TEXT,
        );
        hp.text(
            pos2(x + 14.0, header_rect.top() + 24.0),
            Align2::LEFT_TOP,
            day.day().to_string(),
            fonts::display(22.0),
            if is_today { theme::PRIMARY } else { theme::TEXT },
        );
        if is_today {
            // Same treatment as the month grid's today cell.
            hp.rect_stroke(
                col_rect.shrink(0.5),
                0.0,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 136, 110)),
                StrokeKind::Inside,
            );
        }

        // Column separator.
        if i > 0 {
            hp.line_segment(
                [pos2(x, header_rect.top() + 8.0), pos2(x, header_rect.bottom() - 8.0)],
                Stroke::new(1.0, theme::OUTLINE_VARIANT),
            );
        }

        if response.clicked() {
            *selected_day = Some(day);
        }
    }

    // -- Scrollable time body --
    // Hidden scrollbar so inner content width matches the pinned header exactly.
    egui::ScrollArea::vertical()
        .id_salt("calendar_time_grid")
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            let total_h = HOUR_H * HOURS as f32;
            let (grid_rect, _) =
                ui.allocate_exact_size(vec2(total_w, total_h), Sense::hover());
            let painter = ui.painter_at(grid_rect);

            // Background bands: alternating per hour for readability.
            painter.rect_filled(grid_rect, 0.0, theme::SURFACE_CONTAINER);

            // Hour labels + horizontal lines.
            for h in 0..HOURS {
                let y = grid_rect.top() + h as f32 * HOUR_H;
                painter.text(
                    pos2(grid_rect.left() + HOUR_COL_W - 10.0, y + 4.0),
                    Align2::RIGHT_TOP,
                    format!("{:02}:00", h),
                    FontId::proportional(10.5),
                    theme::DIM_TEXT,
                );
                painter.line_segment(
                    [
                        pos2(grid_rect.left() + HOUR_COL_W, y),
                        pos2(grid_rect.right(), y),
                    ],
                    Stroke::new(1.0, theme::SURFACE_CONTAINER_LOW),
                );
                // Faint half-hour tick.
                let half_y = y + HOUR_H / 2.0;
                painter.line_segment(
                    [
                        pos2(grid_rect.left() + HOUR_COL_W, half_y),
                        pos2(grid_rect.right(), half_y),
                    ],
                    Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(
                            theme::SURFACE_CONTAINER_LOW.r(),
                            theme::SURFACE_CONTAINER_LOW.g(),
                            theme::SURFACE_CONTAINER_LOW.b(),
                            60,
                        ),
                    ),
                );
            }

            // Vertical column separators.
            for i in 0..=days {
                let x = grid_rect.left() + HOUR_COL_W + i as f32 * day_w;
                painter.line_segment(
                    [pos2(x, grid_rect.top()), pos2(x, grid_rect.bottom())],
                    Stroke::new(1.0, theme::OUTLINE_VARIANT),
                );
            }
            // Hour-column right border.
            painter.line_segment(
                [
                    pos2(grid_rect.left() + HOUR_COL_W, grid_rect.top()),
                    pos2(grid_rect.left() + HOUR_COL_W, grid_rect.bottom()),
                ],
                Stroke::new(1.0, theme::OUTLINE_VARIANT),
            );

            // "Now" indicator on today's column.
            let now = chrono::Local::now();
            let now_date = now.date_naive();
            let now_time = now.time();
            for i in 0..days {
                let day = start_date + Duration::days(i as i64);
                if day == now_date {
                    let x = grid_rect.left() + HOUR_COL_W + i as f32 * day_w;
                    let y = grid_rect.top()
                        + (now_time.hour() as f32 + now_time.minute() as f32 / 60.0) * HOUR_H;
                    painter.line_segment(
                        [pos2(x, y), pos2(x + day_w, y)],
                        Stroke::new(1.5, theme::PRIMARY),
                    );
                    painter.circle_filled(pos2(x, y), 3.5, theme::PRIMARY);
                }
            }

            // Event blocks.
            for i in 0..days {
                let day = start_date + Duration::days(i as i64);
                let x = grid_rect.left() + HOUR_COL_W + i as f32 * day_w;
                for event in events.iter().filter(|e| e.date == day) {
                    let start_min =
                        event.start.hour() as f32 * 60.0 + event.start.minute() as f32;
                    let y = grid_rect.top() + (start_min / 60.0) * HOUR_H;
                    let h = ((event.duration_minutes as f32 / 60.0) * HOUR_H).max(22.0);
                    let rect = Rect::from_min_size(
                        pos2(x + 3.0, y + 1.0),
                        vec2(day_w - 6.0, h - 2.0),
                    );
                    draw_event_block(&painter, rect, event);
                }
            }

            // One-shot auto-scroll to 07:00 on first render.
            if !*auto_scrolled {
                let target_y = grid_rect.top() + 7.0 * HOUR_H;
                ui.scroll_to_rect(
                    Rect::from_min_size(pos2(grid_rect.left(), target_y), vec2(10.0, 60.0)),
                    Some(Align::TOP),
                );
                *auto_scrolled = true;
            }
        });
}

fn draw_event_block(painter: &egui::Painter, rect: Rect, event: &Event) {
    let color = event.kind.color();
    let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 38);
    painter.rect(
        rect,
        3.0,
        bg,
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 120),
        ),
        StrokeKind::Inside,
    );
    let stripe = Rect::from_min_size(rect.min, vec2(3.0, rect.height()));
    painter.rect_filled(stripe, 0.0, color);

    // Title fits on the first line; time range on the second if height allows.
    if rect.height() >= 32.0 {
        painter.text(
            pos2(rect.left() + 10.0, rect.top() + 4.0),
            Align2::LEFT_TOP,
            &event.title,
            FontId::proportional(11.5),
            color,
        );
        painter.text(
            pos2(rect.left() + 10.0, rect.top() + 20.0),
            Align2::LEFT_TOP,
            event.time_range(),
            FontId::proportional(9.5),
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 200),
        );
    } else {
        painter.text(
            pos2(rect.left() + 10.0, rect.center().y),
            Align2::LEFT_CENTER,
            format!("{}  ·  {}", event.start.format("%H:%M"), event.title),
            FontId::proportional(11.0),
            color,
        );
    }
}

// ----- shared row used in popup -----

fn event_row(ui: &mut Ui, event: &Event) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 52.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let color = event.kind.color();

    painter.rect(
        rect,
        2.0,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 22),
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90),
        ),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        event.kind.glyph(),
        fonts::icon(18.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 44.0, rect.top() + 9.0),
        Align2::LEFT_TOP,
        &event.title,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 44.0, rect.top() + 27.0),
        Align2::LEFT_TOP,
        format!("{}  ·  {}", event.time_range(), event.kind.label()),
        FontId::proportional(10.5),
        color,
    );
}

// ----- day detail popup (click-through from any view) -----

fn show_day_popup(
    ctx: &Context,
    day: NaiveDate,
    events: &[Event],
    selected_day: &mut Option<NaiveDate>,
) {
    let mut open = true;
    let title = day.format("%A, %B %-d, %Y").to_string();

    egui::Window::new(RichText::new(title).font(fonts::display(16.0)).color(theme::TEXT))
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
            ui.set_min_width(380.0);
            ui.set_max_width(460.0);

            let mut day_events: Vec<&Event> =
                events.iter().filter(|e| e.date == day).collect();
            day_events.sort_by_key(|e| e.start);

            if day_events.is_empty() {
                ui.label(
                    RichText::new("No events scheduled for this day.")
                        .size(13.0)
                        .color(theme::DIM_TEXT),
                );
                return;
            }

            // Group by kind so meetings/tasks/deadlines read as sections.
            for kind in [EventKind::Meeting, EventKind::Task, EventKind::Deadline] {
                let filtered: Vec<&Event> = day_events
                    .iter()
                    .copied()
                    .filter(|e| e.kind == kind)
                    .collect();
                if filtered.is_empty() {
                    continue;
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} ({})", kind_plural(kind), filtered.len()))
                        .size(11.0)
                        .strong()
                        .color(kind.color()),
                );
                for event in filtered {
                    event_row(ui, event);
                    ui.add_space(6.0);
                }
            }
        });

    if !open {
        *selected_day = None;
    }
}

fn kind_plural(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Meeting => "MEETINGS",
        EventKind::Task => "TASKS",
        EventKind::Deadline => "DEADLINES",
    }
}

// ----- legend -----

fn show_legend(ui: &mut Ui) {
    ui.horizontal(|ui| {
        for kind in [EventKind::Meeting, EventKind::Task, EventKind::Deadline] {
            legend_swatch(ui, kind);
            ui.add_space(12.0);
        }
    });
}

fn legend_swatch(ui: &mut Ui, kind: EventKind) {
    ui.horizontal(|ui| {
        let color = kind.color();
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect(
            rect,
            1.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 50),
            Stroke::new(1.0, color),
            StrokeKind::Inside,
        );
        let label = match kind {
            EventKind::Meeting => "Meetings",
            EventKind::Task => "Tasks",
            EventKind::Deadline => "Deadlines",
        };
        ui.label(RichText::new(label).size(11.0).color(theme::DIM_TEXT));
    });
}

// ----- sample data -----

fn sample_events(today: NaiveDate) -> Vec<Event> {
    let t = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
    let e = |offset: i64, kind, title: &str, start: NaiveTime, dur: u32| Event {
        date: today + Duration::days(offset),
        kind,
        title: title.to_string(),
        start,
        duration_minutes: dur,
    };
    vec![
        e(-6, EventKind::Meeting, "Architecture Review", t(10, 0), 60),
        e(-6, EventKind::Task, "Update Schemas", t(14, 0), 90),
        e(-4, EventKind::Deadline, "API Deprecation", t(17, 0), 30),
        e(-2, EventKind::Meeting, "Weekly Standup", t(9, 30), 30),
        e(-1, EventKind::Task, "Draft Intelligence Report", t(13, 0), 120),
        e(0, EventKind::Meeting, "Client: Nexus Corp", t(10, 0), 45),
        e(0, EventKind::Meeting, "Internal Strategy", t(14, 0), 60),
        e(0, EventKind::Task, "Review Graph Nodes", t(16, 30), 60),
        e(1, EventKind::Deadline, "Submit Final Draft", t(17, 0), 30),
        e(3, EventKind::Meeting, "Weekly Standup", t(9, 30), 30),
        e(7, EventKind::Task, "Vault Organization", t(11, 0), 90),
        e(9, EventKind::Meeting, "Product Sync", t(15, 0), 60),
        e(15, EventKind::Deadline, "End of Month Reporting", t(16, 0), 45),
    ]
}
