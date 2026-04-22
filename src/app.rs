use eframe::egui::{self, CentralPanel, Context, Frame, Margin, Vec2};

use crate::{
    calendar, graph, integrations, login, meetings, menu, nav, recordings, settings, tasks, theme,
    upper_bar, vault,
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
    graph: graph::State,
    meetings: meetings::State,
}

impl Default for EpidoteApp {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            email: String::new(),
            password: String::new(),
            selected_nav: menu::Nav::Recordings,
            search: String::new(),
            menu_collapsed: false,
            calendar: calendar::State::default(),
            tasks: tasks::State::default(),
            vault: vault::State::default(),
            graph: graph::State::default(),
            meetings: meetings::State::default(),
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
            Screen::Main => self.show_main(ctx),
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
                menu::Nav::Recordings => recordings::show(ui),
                menu::Nav::Meetings => {
                    if let Some(open) = meetings::show(ui, &mut self.meetings) {
                        self.open(open);
                    }
                }
                menu::Nav::Tasks => tasks::show(ui, &mut self.tasks),
                menu::Nav::Vault => vault::show(ui, &mut self.vault),
                menu::Nav::Graph => {
                    if let Some(open) = graph::show(ui, &mut self.graph) {
                        self.open(open);
                    }
                }
                menu::Nav::Calendar => calendar::show(ui, &mut self.calendar),
                menu::Nav::Integrations => integrations::show(ui),
                menu::Nav::Settings => settings::show(ui),
            });
    }

    fn open(&mut self, open: nav::Open) {
        match open {
            nav::Open::Vault(path) => {
                self.vault.open(path);
                self.selected_nav = menu::Nav::Vault;
            }
            nav::Open::Task(id) => {
                self.tasks.select(id);
                self.selected_nav = menu::Nav::Tasks;
            }
        }
    }
}
