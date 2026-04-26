use eframe::egui::{
    self, pos2, vec2, Align2, CentralPanel, Color32, Context, Frame, Margin, Sense, Stroke, Vec2,
};

use crate::{
    calendar, login, meetings, menu, nav, proposals, settings, tasks, theme, today, upper_bar,
    vault,
};

pub struct EpidoteApp {
    screen: Screen,
    email: String,
    password: String,
    selected_nav: menu::Nav,
    search: String,
    menu_collapsed: bool,
    calendar: calendar::State,
    tasks: tasks::State,
    vault: vault::State,
    meetings: meetings::State,
    proposals: proposals::State,
    cmdk: CmdK,
}

#[derive(Default)]
struct CmdK {
    open: bool,
    query: String,
}

impl Default for EpidoteApp {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            email: String::new(),
            password: String::new(),
            selected_nav: menu::Nav::Today,
            search: String::new(),
            menu_collapsed: false,
            calendar: calendar::State::default(),
            tasks: tasks::State::default(),
            vault: vault::State::default(),
            meetings: meetings::State::default(),
            proposals: proposals::State::default(),
            cmdk: CmdK::default(),
        }
    }
}

#[derive(Default)]
enum Screen {
    #[default]
    Login,
    Main,
}

impl eframe::App for EpidoteApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.visuals = egui::Visuals::dark();
            style.visuals.panel_fill = theme::BACKGROUND;
            style.visuals.window_fill = theme::BACKGROUND;
            style.visuals.override_text_color = Some(theme::TEXT);
            style.spacing.item_spacing = Vec2::new(12.0, 12.0);
            style.spacing.button_padding = Vec2::new(18.0, 14.0);
        });

        match self.screen {
            Screen::Login => {
                if login::show(ctx, &mut self.email, &mut self.password) {
                    self.screen = Screen::Main;
                }
            }
            Screen::Main => {
                // Cmd-K toggle is captured *before* rendering so the palette
                // can open from any page.
                ctx.input(|i| {
                    let mod_ = i.modifiers.command || i.modifiers.ctrl;
                    if mod_ && i.key_pressed(egui::Key::K) {
                        self.cmdk.open = !self.cmdk.open;
                        if self.cmdk.open {
                            self.cmdk.query.clear();
                        }
                    }
                    if i.key_pressed(egui::Key::Escape) && self.cmdk.open {
                        self.cmdk.open = false;
                    }
                });

                self.show_main(ctx);

                if self.cmdk.open {
                    if let Some(open) = render_cmdk(ctx, &mut self.cmdk) {
                        self.cmdk.open = false;
                        self.open(open);
                    }
                }
            }
        }
    }
}

impl EpidoteApp {
    fn show_main(&mut self, ctx: &Context) {
        if !self.menu_collapsed {
            egui::SidePanel::left("side_nav")
                .resizable(false)
                .exact_width(256.0)
                .frame(
                    Frame::default()
                        .fill(theme::SURFACE_CONTAINER_LOW)
                        .inner_margin(Margin::same(16)),
                )
                .show_separator_line(false)
                .show(ctx, |ui| {
                    menu::show(ui, &mut self.selected_nav, &mut self.menu_collapsed);
                });
        }

        egui::TopBottomPanel::top("top_bar")
            .exact_height(64.0)
            .frame(
                Frame::default()
                    .fill(theme::BACKGROUND)
                    .inner_margin(Margin::symmetric(24, 0)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                upper_bar::show(ui, &mut self.search, &mut self.menu_collapsed);
            });

        CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(theme::BACKGROUND)
                    .inner_margin(Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| match self.selected_nav {
                menu::Nav::Today => {
                    if let Some(open) = today::show(ui) {
                        self.open(open);
                    }
                }
                menu::Nav::Meetings => {
                    if let Some(open) = meetings::show(ui, &mut self.meetings) {
                        self.open(open);
                    }
                }
                menu::Nav::Tasks => {
                    if let Some(open) = tasks::show(ui, &mut self.tasks) {
                        self.open(open);
                    }
                }
                menu::Nav::Proposals => {
                    if let Some(open) = proposals::show(ui, &mut self.proposals) {
                        self.open(open);
                    }
                }
                menu::Nav::Vault => {
                    if let Some(open) = vault::show(ui, &mut self.vault) {
                        self.open(open);
                    }
                }
                menu::Nav::Calendar => {
                    if let Some(open) = calendar::show(ui, &mut self.calendar) {
                        self.open(open);
                    }
                }
                menu::Nav::Settings => settings::show(ui),
            });
    }

}

// ----- Cmd-K palette -----

struct CmdKEntry {
    kind: &'static str,
    glyph: &'static str,
    title: String,
    sub: String,
    target: nav::Open,
}

fn render_cmdk(ctx: &Context, state: &mut CmdK) -> Option<nav::Open> {
    let mut chosen: Option<nav::Open> = None;

    // Dim backdrop covering the whole window.
    egui::Area::new(egui::Id::new("cmdk_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let response = ui.allocate_rect(screen, Sense::click());
            ui.painter().rect_filled(
                screen,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            );
            if response.clicked() {
                state.open = false;
            }
        });

    let entries = build_cmdk_entries(&state.query);

    egui::Area::new(egui::Id::new("cmdk_window"))
        .order(egui::Order::Tooltip)
        .anchor(Align2::CENTER_TOP, [0.0, 110.0])
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(theme::SURFACE_CONTAINER)
                .stroke(Stroke::new(1.0, theme::OUTLINE))
                .inner_margin(Margin::same(0))
                .show(ui, |ui| {
                    ui.set_width(560.0);

                    // Input row
                    let input_h = 48.0;
                    let (input_rect, _) =
                        ui.allocate_exact_size(vec2(560.0, input_h), Sense::hover());
                    ui.painter().line_segment(
                        [input_rect.left_bottom(), input_rect.right_bottom()],
                        Stroke::new(1.0, theme::OUTLINE_VARIANT),
                    );
                    // Search icon
                    ui.painter().text(
                        pos2(input_rect.left() + 16.0, input_rect.center().y),
                        Align2::LEFT_CENTER,
                        crate::icons::SEARCH,
                        crate::fonts::icon(15.0),
                        theme::DIM_TEXT,
                    );
                    // Text field
                    let field_rect = egui::Rect::from_min_max(
                        pos2(input_rect.left() + 40.0, input_rect.top() + 8.0),
                        pos2(input_rect.right() - 60.0, input_rect.bottom() - 8.0),
                    );
                    let mut field_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(field_rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    let edit = field_ui.add(
                        egui::TextEdit::singleline(&mut state.query)
                            .hint_text("Search or type a command…")
                            .frame(false)
                            .desired_width(f32::INFINITY)
                            .font(egui::FontId::proportional(14.0)),
                    );
                    edit.request_focus();
                    if edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(first) = entries.first() {
                            chosen = Some(first.target.clone());
                        }
                    }
                    // ESC pill
                    ui.painter().text(
                        pos2(input_rect.right() - 18.0, input_rect.center().y),
                        Align2::RIGHT_CENTER,
                        "ESC",
                        egui::FontId::monospace(10.0),
                        theme::DIM_TEXT,
                    );

                    // Results
                    if entries.is_empty() {
                        let (empty_rect, _) =
                            ui.allocate_exact_size(vec2(560.0, 60.0), Sense::hover());
                        ui.painter().text(
                            empty_rect.center(),
                            Align2::CENTER_CENTER,
                            "No matches.",
                            egui::FontId::proportional(12.0),
                            theme::DIM_TEXT,
                        );
                    } else {
                        for entry in &entries {
                            let row_h = 44.0;
                            let (row_rect, response) =
                                ui.allocate_exact_size(vec2(560.0, row_h), Sense::click());
                            let painter = ui.painter_at(row_rect);
                            if response.hovered() {
                                painter.rect_filled(row_rect, 0.0, theme::SURFACE_HIGH);
                            }
                            // Kind glyph
                            painter.text(
                                pos2(row_rect.left() + 18.0, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                entry.glyph,
                                crate::fonts::icon(14.0),
                                theme::SOFT_TEXT,
                            );
                            // Title + sub
                            painter.text(
                                pos2(row_rect.left() + 42.0, row_rect.top() + 8.0),
                                Align2::LEFT_TOP,
                                &entry.title,
                                egui::FontId::proportional(13.0),
                                theme::TEXT,
                            );
                            painter.text(
                                pos2(row_rect.left() + 42.0, row_rect.top() + 25.0),
                                Align2::LEFT_TOP,
                                &entry.sub,
                                egui::FontId::monospace(10.5),
                                theme::DIM_TEXT,
                            );
                            // Kind tag right-aligned
                            painter.text(
                                pos2(row_rect.right() - 16.0, row_rect.center().y),
                                Align2::RIGHT_CENTER,
                                entry.kind,
                                egui::FontId::monospace(9.5),
                                theme::DIM_TEXT,
                            );
                            if response.clicked() {
                                chosen = Some(entry.target.clone());
                            }
                        }
                    }
                });
        });

    chosen
}

fn build_cmdk_entries(query: &str) -> Vec<CmdKEntry> {
    use crate::icons;
    let q = query.trim().to_lowercase();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let vault = std::path::PathBuf::from(home).join("Documents/Epidote/Vault");
    let path = |rel: &str| vault.join(rel);

    // Static catalog of jumpable entities. Mirrors the seeded data we ship in
    // Tasks/Meetings/Vault. Future phases will read these from a shared store.
    let catalog: Vec<CmdKEntry> = vec![
        CmdKEntry {
            kind: "MEETING",
            glyph: icons::GRAPHIC_EQ,
            title: "System Architecture Sync".into(),
            sub: "live · Apr 22, 2026".into(),
            target: nav::Open::Meeting("System Architecture Sync".into()),
        },
        CmdKEntry {
            kind: "MEETING",
            glyph: icons::GRAPHIC_EQ,
            title: "Q3 Roadmap Planning".into(),
            sub: "processing · Apr 22, 2026".into(),
            target: nav::Open::Meeting("Q3 Roadmap Planning".into()),
        },
        CmdKEntry {
            kind: "MEETING",
            glyph: icons::GRAPHIC_EQ,
            title: "Security Audit Review".into(),
            sub: "Oct 20, 2026".into(),
            target: nav::Open::Meeting("Security Audit Review".into()),
        },
        CmdKEntry {
            kind: "TASK",
            glyph: icons::CHECK_CIRCLE,
            title: "Determine optimal chunking strategy for vector embeddings".into(),
            sub: "TSK-092 · active".into(),
            target: nav::Open::Task("TSK-092".into()),
        },
        CmdKEntry {
            kind: "TASK",
            glyph: icons::CHECK_CIRCLE,
            title: "Migrate legacy logging to Datadog cluster".into(),
            sub: "TSK-088 · blocked".into(),
            target: nav::Open::Task("TSK-088".into()),
        },
        CmdKEntry {
            kind: "TASK",
            glyph: icons::CHECK_CIRCLE,
            title: "Review Redis caching strategy for high-frequency reads".into(),
            sub: "TSK-104 · triage".into(),
            target: nav::Open::Task("TSK-104".into()),
        },
        CmdKEntry {
            kind: "NOTE",
            glyph: icons::DESCRIPTION,
            title: "Embedding Architecture".into(),
            sub: "Architecture".into(),
            target: nav::Open::Vault(path("Architecture/Embedding Architecture.md")),
        },
        CmdKEntry {
            kind: "NOTE",
            glyph: icons::DESCRIPTION,
            title: "Chunking Benchmarks".into(),
            sub: "Architecture".into(),
            target: nav::Open::Vault(path("Architecture/Chunking Benchmarks.md")),
        },
        CmdKEntry {
            kind: "NOTE",
            glyph: icons::DESCRIPTION,
            title: "Caching RFC v2".into(),
            sub: "Architecture".into(),
            target: nav::Open::Vault(path("Architecture/Caching RFC v2.md")),
        },
        CmdKEntry {
            kind: "PROPOSAL",
            glyph: icons::AUTO_AWESOME,
            title: "Use 512-token windows with 64-token overlap as the RAG default".into(),
            sub: "from System Architecture Sync · 82%".into(),
            target: nav::Open::Proposal("prop-chunking".into()),
        },
        CmdKEntry {
            kind: "PROPOSAL",
            glyph: icons::AUTO_AWESOME,
            title: "Generated: Migration Manifesto v1.md".into(),
            sub: "from Q3 Data Architecture · 79%".into(),
            target: nav::Open::Proposal("prop-doc-migration".into()),
        },
    ];

    if q.is_empty() {
        return catalog.into_iter().take(8).collect();
    }
    catalog
        .into_iter()
        .filter(|e| {
            e.title.to_lowercase().contains(&q)
                || e.sub.to_lowercase().contains(&q)
                || e.kind.to_lowercase().contains(&q)
        })
        .collect()
}

impl EpidoteApp {
    fn open(&mut self, open: nav::Open) {
        match open {
            nav::Open::Tab(tab) => {
                self.selected_nav = tab;
            }
            nav::Open::Vault(path) => {
                self.vault.open(path);
                self.selected_nav = menu::Nav::Vault;
            }
            nav::Open::Task(id) => {
                self.tasks.select(id);
                self.selected_nav = menu::Nav::Tasks;
            }
            nav::Open::Meeting(title) => {
                self.meetings.select(&title);
                self.selected_nav = menu::Nav::Meetings;
            }
            nav::Open::Proposal(id) => {
                self.proposals.select(&id);
                self.selected_nav = menu::Nav::Proposals;
            }
        }
    }
}
