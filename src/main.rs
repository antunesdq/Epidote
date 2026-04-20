use std::sync::Arc;

use eframe::egui::{
    self, pos2, vec2, Align, Button, CentralPanel, Color32, Context, FontFamily, FontId, Frame,
    Key, Layout, Margin, Rect, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
};

const BACKGROUND: Color32 = Color32::from_rgb(0x13, 0x13, 0x13);
const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(0x1b, 0x1b, 0x1c);
const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
const SURFACE_HIGHEST: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
const TEXT: Color32 = Color32::from_rgb(0xe5, 0xe2, 0xe1);
const DIM_TEXT: Color32 = Color32::from_rgb(0x5b, 0x64, 0x61);
const PRIMARY: Color32 = Color32::from_rgb(0x00, 0xff, 0x88);
const PRIMARY_SOFT: Color32 = Color32::from_rgb(0x60, 0xff, 0x99);
const BUTTON_TEXT: Color32 = Color32::from_rgb(0x00, 0x21, 0x0c);
const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0x2a, 0x33, 0x2c);
const OUTLINE: Color32 = Color32::from_rgb(0x3b, 0x4b, 0x3d);

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Epidote - Technical Intelligence")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Epidote",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(EpidoteApp::default()))
        }),
    )
}

/// Registers the three bundled typefaces and exposes them via:
/// - `FontFamily::Proportional` — Inter (primary) with Material Symbols as a glyph
///   fallback so PUA codepoints embedded in regular labels still render.
/// - `FontFamily::Name("display")` — Space Grotesk for brand/headline text.
/// - `FontFamily::Name("icons")` — Material Symbols, addressed by the constants
///   in the `icon` module.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Inter".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/InterVariable.ttf"
        ))),
    );
    fonts.font_data.insert(
        "SpaceGrotesk".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/SpaceGrotesk.ttf"
        ))),
    );
    fonts.font_data.insert(
        "MaterialSymbols".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/MaterialSymbolsOutlined.ttf"
        ))),
    );

    let prop = fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default();
    prop.insert(0, "Inter".to_owned());
    prop.push("MaterialSymbols".to_owned());

    fonts.families.insert(
        FontFamily::Name("display".into()),
        vec!["SpaceGrotesk".to_owned(), "Inter".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("icons".into()),
        vec!["MaterialSymbols".to_owned()],
    );

    ctx.set_fonts(fonts);
}

fn icon_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("icons".into()))
}

fn display_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("display".into()))
}

/// Material Symbols Outlined codepoints. Values are the standard PUA codepoints
/// shared with Material Icons — see fonts.google.com/icons for the catalog.
mod icon {
    pub const HUB: &str = "\u{e9f4}";
    pub const MIC: &str = "\u{e029}";
    pub const GROUPS: &str = "\u{f233}";
    pub const CHECK_CIRCLE: &str = "\u{e86c}";
    pub const LOCK: &str = "\u{e897}";
    pub const ACCOUNT_TREE: &str = "\u{e97a}";
    pub const CALENDAR_TODAY: &str = "\u{e935}";
    pub const EXTENSION: &str = "\u{e87b}";
    pub const SETTINGS: &str = "\u{e8b8}";
    pub const SEARCH: &str = "\u{e8b6}";
    pub const NOTIFICATIONS: &str = "\u{e7f4}";
    pub const HELP: &str = "\u{e887}";
    pub const ADD: &str = "\u{e145}";
    pub const ARROW_FORWARD: &str = "\u{e5c8}";
    pub const VPN_KEY: &str = "\u{e0da}";
}

struct EpidoteApp {
    screen: Screen,
    email: String,
    password: String,
    selected_nav: Nav,
    search: String,
}

impl Default for EpidoteApp {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            email: String::new(),
            password: String::new(),
            selected_nav: Nav::Record,
            search: String::new(),
        }
    }
}

#[derive(Default)]
enum Screen {
    #[default]
    Login,
    Main,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Nav {
    #[default]
    Record,
    Meetings,
    Tasks,
    Vault,
    Graph,
    Calendar,
    Integrations,
    Settings,
}

impl Nav {
    fn label(self) -> &'static str {
        match self {
            Nav::Record => "Record",
            Nav::Meetings => "Meetings",
            Nav::Tasks => "Tasks",
            Nav::Vault => "Vault",
            Nav::Graph => "Graph",
            Nav::Calendar => "Calendar",
            Nav::Integrations => "Integrations",
            Nav::Settings => "Settings",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Nav::Record => icon::MIC,
            Nav::Meetings => icon::GROUPS,
            Nav::Tasks => icon::CHECK_CIRCLE,
            Nav::Vault => icon::LOCK,
            Nav::Graph => icon::ACCOUNT_TREE,
            Nav::Calendar => icon::CALENDAR_TODAY,
            Nav::Integrations => icon::EXTENSION,
            Nav::Settings => icon::SETTINGS,
        }
    }
}

impl eframe::App for EpidoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.visuals = egui::Visuals::dark();
            style.visuals.panel_fill = BACKGROUND;
            style.visuals.window_fill = BACKGROUND;
            style.visuals.override_text_color = Some(TEXT);
            style.spacing.item_spacing = Vec2::new(12.0, 12.0);
            style.spacing.button_padding = Vec2::new(18.0, 14.0);
        });

        match self.screen {
            Screen::Login => self.show_login_screen(ctx),
            Screen::Main => self.show_main_screen(ctx),
        }
    }
}

impl EpidoteApp {
    fn show_login_screen(&mut self, ctx: &Context) {
        self.paint_background(ctx);

        CentralPanel::default()
            .frame(Frame::default().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let panel_width = ui.available_width().min(448.0);
                    let top_padding = ((ui.available_height() - 560.0).max(32.0)) * 0.4;

                    ui.add_space(top_padding);

                    ui.allocate_ui_with_layout(
                        Vec2::new(panel_width, ui.available_height()),
                        Layout::top_down(Align::Min),
                        |ui| {
                            self.show_brand_header(ui);
                            ui.add_space(28.0);
                            self.show_login_card(ui);
                            ui.add_space(18.0);
                            self.show_footer(ui);
                        },
                    );
                });
            });
    }

    fn show_main_screen(&mut self, ctx: &Context) {
        egui::SidePanel::left("side_nav")
            .resizable(false)
            .exact_width(256.0)
            .frame(
                Frame::default()
                    .fill(SURFACE_CONTAINER_LOW)
                    .inner_margin(Margin::same(16)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                self.show_sidebar(ui);
            });

        egui::TopBottomPanel::top("top_bar")
            .exact_height(64.0)
            .frame(
                Frame::default()
                    .fill(BACKGROUND)
                    .inner_margin(Margin::symmetric(24, 0)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                self.show_top_bar(ui);
            });

        CentralPanel::default()
            .frame(Frame::default().fill(BACKGROUND))
            .show(ctx, |_ui| {});
    }

    fn show_top_bar(&mut self, ui: &mut Ui) {
        let bar_rect = ui.max_rect();
        let center_y = bar_rect.center().y;

        // Centered search bar (max-w-md ≈ 448 in Tailwind).
        let search_h = 36.0;
        let search_w = 448.0_f32.min((bar_rect.width() - 320.0).max(240.0));
        let search_rect = Rect::from_center_size(
            pos2(bar_rect.center().x, center_y),
            vec2(search_w, search_h),
        );

        ui.scope(|ui| {
            let v = &mut ui.style_mut().visuals;
            v.widgets.inactive.bg_fill = SURFACE_HIGH;
            v.widgets.inactive.weak_bg_fill = SURFACE_HIGH;
            v.widgets.inactive.bg_stroke = Stroke::NONE;
            v.widgets.inactive.fg_stroke.color = TEXT;
            v.widgets.hovered.bg_fill = SURFACE_HIGH;
            v.widgets.hovered.weak_bg_fill = SURFACE_HIGH;
            v.widgets.hovered.bg_stroke = Stroke::NONE;
            v.widgets.hovered.fg_stroke.color = TEXT;
            v.widgets.active.bg_fill = SURFACE_HIGHEST;
            v.widgets.active.weak_bg_fill = SURFACE_HIGHEST;
            v.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY_SOFT);
            v.widgets.active.fg_stroke.color = TEXT;
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 255, 136, 80);

            ui.put(
                search_rect,
                TextEdit::singleline(&mut self.search)
                    .hint_text("Search tasks, meetings, architecture...")
                    .margin(Margin {
                        left: 34,
                        right: 12,
                        top: 8,
                        bottom: 8,
                    }),
            );
        });

        // Magnifier glyph inside the search pill.
        ui.painter().text(
            pos2(search_rect.left() + 16.0, center_y),
            egui::Align2::CENTER_CENTER,
            icon::SEARCH,
            icon_font(18.0),
            DIM_TEXT,
        );

        // Right-aligned cluster: notifications, help, avatar.
        let avatar_d = 32.0;
        let btn_d = 36.0;
        let gap = 8.0;

        let avatar_rect = Rect::from_center_size(
            pos2(bar_rect.right() - avatar_d / 2.0, center_y),
            Vec2::splat(avatar_d),
        );
        let painter = ui.painter();
        painter.circle_filled(avatar_rect.center(), avatar_d / 2.0, SURFACE_HIGHEST);
        painter.circle_stroke(
            avatar_rect.center(),
            avatar_d / 2.0,
            Stroke::new(1.0, OUTLINE_VARIANT),
        );

        let help_rect = Rect::from_center_size(
            pos2(avatar_rect.left() - gap - btn_d / 2.0, center_y),
            Vec2::splat(btn_d),
        );
        let notif_rect = Rect::from_center_size(
            pos2(help_rect.left() - gap - btn_d / 2.0, center_y),
            Vec2::splat(btn_d),
        );

        self.top_icon_button(ui, notif_rect, "notif", icon::NOTIFICATIONS);
        self.top_icon_button(ui, help_rect, "help", icon::HELP);
    }

    fn top_icon_button(
        &self,
        ui: &mut Ui,
        rect: Rect,
        id: &'static str,
        glyph: &'static str,
    ) -> egui::Response {
        let response = ui.interact(rect, egui::Id::new(("top_bar_btn", id)), Sense::click());
        let painter = ui.painter_at(rect);
        if response.hovered() {
            painter.rect_filled(rect, 4.0, SURFACE_HIGHEST);
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            icon_font(20.0),
            TEXT,
        );
        response
    }

    fn show_sidebar(&mut self, ui: &mut Ui) {
        self.show_sidebar_header(ui);
        ui.add_space(16.0);

        let top_items = [
            Nav::Record,
            Nav::Meetings,
            Nav::Tasks,
            Nav::Vault,
            Nav::Graph,
            Nav::Calendar,
            Nav::Integrations,
        ];
        for nav in top_items {
            self.nav_item(ui, nav);
            ui.add_space(2.0);
        }

        // Pin Settings + CTA to the bottom of the rail.
        let bottom_block = 40.0 /* settings */ + 16.0 /* gap */ + 48.0 /* cta */;
        let spacer = (ui.available_height() - bottom_block).max(8.0);
        ui.add_space(spacer);

        self.nav_item(ui, Nav::Settings);
        ui.add_space(16.0);
        self.record_new_button(ui);
    }

    fn show_sidebar_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            let (avatar_rect, _) =
                ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(avatar_rect, 4.0, SURFACE_HIGHEST);
            painter.text(
                avatar_rect.center(),
                egui::Align2::CENTER_CENTER,
                icon::HUB,
                icon_font(24.0),
                PRIMARY,
            );

            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Epidote")
                        .font(display_font(20.0))
                        .strong()
                        .color(PRIMARY),
                );
                ui.add_space(-4.0);
                ui.label(
                    RichText::new("Technical Intelligence")
                        .size(11.0)
                        .color(DIM_TEXT),
                );
            });
        });
    }

    fn nav_item(&mut self, ui: &mut Ui, nav: Nav) {
        let active = self.selected_nav == nav;
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(vec2(width, 40.0), Sense::click());
        let painter = ui.painter_at(rect);
        let hovered = response.hovered();

        let bg = if active || hovered {
            SURFACE_CONTAINER
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(rect, 2.0, bg);

        if active {
            let accent = Rect::from_min_size(rect.min, vec2(2.0, rect.height()));
            painter.rect_filled(accent, 0.0, PRIMARY);
        }

        let text_color = if active || hovered {
            PRIMARY
        } else {
            Color32::from_rgba_unmultiplied(TEXT.r(), TEXT.g(), TEXT.b(), 153)
        };

        painter.text(
            pos2(rect.left() + 14.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            nav.glyph(),
            icon_font(18.0),
            text_color,
        );
        painter.text(
            pos2(rect.left() + 40.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            nav.label(),
            FontId::proportional(13.0),
            text_color,
        );

        if response.clicked() {
            self.selected_nav = nav;
        }
    }

    fn record_new_button(&self, ui: &mut Ui) {
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(vec2(width, 48.0), Sense::click());
        let painter = ui.painter_at(rect);
        let fill = if response.hovered() { PRIMARY_SOFT } else { PRIMARY };
        painter.rect_filled(rect, 2.0, fill);

        // "+" glyph followed by label; measured so the pair centers inside the pill.
        let icon_size = 18.0;
        let label = "Record New";
        let label_font = display_font(13.5);
        let label_galley =
            painter.layout_no_wrap(label.to_owned(), label_font.clone(), BUTTON_TEXT);
        let spacing = 8.0;
        let total = icon_size + spacing + label_galley.size().x;
        let cursor_x = rect.center().x - total / 2.0;

        painter.text(
            pos2(cursor_x + icon_size / 2.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon::ADD,
            icon_font(icon_size),
            BUTTON_TEXT,
        );
        painter.text(
            pos2(cursor_x + icon_size + spacing, rect.center().y - label_galley.size().y / 2.0),
            egui::Align2::LEFT_TOP,
            label,
            label_font,
            BUTTON_TEXT,
        );
    }

    fn paint_background(&self, ctx: &Context) {
        let rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(rect, 0.0, BACKGROUND);

        // Faint emerald wash in the upper portion of the viewport. The HTML's
        // effect is a single 800px disk with opacity:0.05 + blur(120px); we
        // recreate the soft falloff with a radial mesh of concentric rings
        // whose per-vertex alpha fades smoothly to zero.
        let center = pos2(
            rect.left() + rect.width() * 0.35,
            rect.top() + rect.height() * 0.12,
        );
        let radius = rect.width().max(rect.height()) * 0.95;
        paint_radial_gradient(
            &painter,
            center,
            radius,
            Color32::from_rgb(0x00, 0xff, 0x88),
            // (fraction-of-radius, alpha) — tuned to read as a faint wash, not a disk.
            &[
                (0.00, 4),
                (0.18, 3),
                (0.38, 2),
                (0.62, 1),
                (0.82, 0),
                (1.00, 0),
            ],
        );
    }

    fn show_brand_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon::HUB,
                icon_font(32.0),
                PRIMARY,
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Epidote")
                    .font(display_font(30.0))
                    .strong()
                    .color(PRIMARY),
            );
        });
    }

    fn show_login_card(&mut self, ui: &mut Ui) {
        Frame::default()
            .fill(SURFACE_CONTAINER)
            .stroke(Stroke::new(1.0, OUTLINE_VARIANT))
            .corner_radius(2)
            .inner_margin(Margin::same(32))
            .show(ui, |ui| {
                let mut open_main = false;

                // Floating "Auth_v2.4" tag pinned to the top-right of the card.
                let header_rect = ui.available_rect_before_wrap();
                let tag_pos = pos2(header_rect.right(), header_rect.top());
                ui.painter().text(
                    tag_pos,
                    egui::Align2::RIGHT_TOP,
                    "AUTH_V2.4",
                    egui::FontId::monospace(10.0),
                    DIM_TEXT,
                );

                ui.label(
                    RichText::new("Secure Access")
                        .font(display_font(24.0))
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(-4.0);
                ui.label(
                    RichText::new("Authenticate to access the technical intelligence archive.")
                        .size(13.0)
                        .color(DIM_TEXT),
                );

                ui.add_space(20.0);

                ui.scope(|ui| {
                    let visuals = &mut ui.style_mut().visuals;
                    visuals.widgets.inactive.bg_fill = SURFACE_HIGH;
                    visuals.widgets.inactive.weak_bg_fill = SURFACE_HIGH;
                    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                    visuals.widgets.inactive.fg_stroke.color = TEXT;
                    visuals.widgets.hovered.bg_fill = SURFACE_HIGH;
                    visuals.widgets.hovered.weak_bg_fill = SURFACE_HIGH;
                    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                    visuals.widgets.hovered.fg_stroke.color = TEXT;
                    visuals.widgets.active.bg_fill = SURFACE_HIGHEST;
                    visuals.widgets.active.weak_bg_fill = SURFACE_HIGHEST;
                    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY_SOFT);
                    visuals.widgets.active.fg_stroke.color = TEXT;
                    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 255, 136, 80);

                    ui.label(
                        RichText::new("IDENTIFIER (EMAIL)")
                            .size(10.5)
                            .strong()
                            .color(DIM_TEXT),
                    );
                    ui.add_space(2.0);

                    ui.add_sized(
                        [ui.available_width(), 44.0],
                        TextEdit::singleline(&mut self.email)
                            .hint_text("operative@epidote.io")
                            .margin(Margin::symmetric(12, 12)),
                    );

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("ACCESS KEY")
                                .size(10.5)
                                .strong()
                                .color(DIM_TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("Recover Key")
                                    .size(11.5)
                                    .color(PRIMARY),
                            );
                        });
                    });
                    ui.add_space(2.0);

                    let password_response = ui.add_sized(
                        [ui.available_width(), 44.0],
                        TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("••••••••••••••••")
                            .margin(Margin::symmetric(12, 12)),
                    );

                    if password_response.lost_focus()
                        && ui.input(|input| input.key_pressed(Key::Enter))
                    {
                        open_main = true;
                    }
                });

                ui.add_space(18.0);

                // Material Symbols "arrow_forward" (U+E5C8) resolves via the
                // fallback slot in FontFamily::Proportional, so it inherits the
                // label's size without a second draw call.
                let primary_button = Button::new(
                    RichText::new(format!("Initialize Session  {}", icon::ARROW_FORWARD))
                        .size(13.5)
                        .strong()
                        .color(BUTTON_TEXT),
                )
                .fill(PRIMARY)
                .stroke(Stroke::NONE)
                .corner_radius(2)
                .min_size(Vec2::new(ui.available_width(), 48.0));

                if ui.add(primary_button).clicked() {
                    open_main = true;
                }

                ui.add_space(14.0);
                self.draw_hairline(ui);
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("SSO Protocol")
                            .size(12.0)
                            .color(DIM_TEXT),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.sso_button(ui).clicked() {
                            open_main = true;
                        }
                    });
                });

                if open_main {
                    self.screen = Screen::Main;
                }
            });
    }

    fn draw_hairline(&self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        let y = rect.top();
        let line = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), 1.0));
        ui.painter().rect_filled(line, 0.0, OUTLINE_VARIANT);
        ui.advance_cursor_after_rect(line);
    }

    /// "Authenticate" secondary action: outlined pill with a drawn key glyph.
    fn sso_button(&self, ui: &mut Ui) -> egui::Response {
        let desired = vec2(130.0, 30.0);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
        let painter = ui.painter_at(rect);

        let (fill, stroke_color) = if response.hovered() {
            (SURFACE_HIGHEST, OUTLINE)
        } else {
            (Color32::TRANSPARENT, OUTLINE_VARIANT)
        };
        painter.rect(
            rect,
            2.0,
            fill,
            Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        painter.text(
            pos2(rect.left() + 14.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon::VPN_KEY,
            icon_font(16.0),
            TEXT,
        );
        painter.text(
            pos2(rect.left() + 26.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Authenticate",
            FontId::proportional(12.0),
            TEXT,
        );

        response
    }

    fn show_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Documentation").size(11.5).color(DIM_TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("System Status").size(11.5).color(DIM_TEXT));
            });
        });
    }
}

/// Paints a soft radial gradient by building a triangulated fan of concentric
/// rings whose vertex alphas interpolate between the supplied stops. Each stop
/// is `(fraction_of_radius, alpha)` in ascending order; alpha is 0–255.
fn paint_radial_gradient(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
    stops: &[(f32, u8)],
) {
    assert!(stops.len() >= 2, "need at least an inner and outer stop");
    const SEGMENTS: usize = 96;

    let mut mesh = egui::Mesh::default();
    let mut ring_starts: Vec<u32> = Vec::with_capacity(stops.len());

    let with_alpha = |a: u8| Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a);

    // First stop: a single center vertex; later stops: a full ring of SEGMENTS vertices.
    for (i, &(frac, alpha)) in stops.iter().enumerate() {
        ring_starts.push(mesh.vertices.len() as u32);
        if i == 0 {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: center,
                uv: egui::epaint::WHITE_UV,
                color: with_alpha(alpha),
            });
        } else {
            let r = radius * frac;
            let col = with_alpha(alpha);
            for s in 0..SEGMENTS {
                let angle = (s as f32) * std::f32::consts::TAU / SEGMENTS as f32;
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: pos2(center.x + r * angle.cos(), center.y + r * angle.sin()),
                    uv: egui::epaint::WHITE_UV,
                    color: col,
                });
            }
        }
    }

    // Fan from the center vertex to the first ring.
    let first_ring = ring_starts[1];
    for s in 0..SEGMENTS as u32 {
        let next = (s + 1) % SEGMENTS as u32;
        mesh.indices.extend_from_slice(&[0, first_ring + s, first_ring + next]);
    }

    // Quad strips between each pair of adjacent rings.
    for ring in 1..ring_starts.len() - 1 {
        let inner = ring_starts[ring];
        let outer = ring_starts[ring + 1];
        for s in 0..SEGMENTS as u32 {
            let next = (s + 1) % SEGMENTS as u32;
            let a = inner + s;
            let b = inner + next;
            let c = outer + s;
            let d = outer + next;
            mesh.indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    painter.add(egui::Shape::mesh(mesh));
}
