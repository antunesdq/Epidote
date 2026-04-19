use eframe::egui::{
    self, pos2, Align, Button, CentralPanel, Color32, Context, Frame, Key, Layout, Margin,
    RichText, Stroke, TextEdit, Ui, Vec2,
};

const BACKGROUND: Color32 = Color32::from_rgb(0x13, 0x13, 0x13);
const SURFACE: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
const SURFACE_HIGHEST: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
const TEXT: Color32 = Color32::from_rgb(0xe5, 0xe2, 0xe1);
const MUTED_TEXT: Color32 = Color32::from_rgb(0xd7, 0xe0, 0xdc);
const PRIMARY: Color32 = Color32::from_rgb(0x00, 0xff, 0x88);
const PRIMARY_SOFT: Color32 = Color32::from_rgb(0x60, 0xff, 0x99);
const BUTTON_TEXT: Color32 = Color32::from_rgb(0x00, 0x21, 0x0c);
const OUTLINE: Color32 = Color32::from_rgb(0x3b, 0x4b, 0x3d);

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Epidote")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Epidote",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(EpidoteApp::default()))
        }),
    )
}

struct EpidoteApp {
    screen: Screen,
    email: String,
    password: String,
}

impl Default for EpidoteApp {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            email: String::new(),
            password: String::new(),
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
                    let panel_width = ui.available_width().min(470.0);
                    let top_padding = ((ui.available_height() - 520.0).max(32.0)) * 0.45;

                    ui.add_space(top_padding);

                    ui.allocate_ui_with_layout(
                        Vec2::new(panel_width, ui.available_height()),
                        Layout::top_down(Align::Min),
                        |ui| {
                            self.show_brand_header(ui);
                            ui.add_space(20.0);
                            self.show_login_card(ui);
                            ui.add_space(16.0);
                            self.show_footer(ui);
                        },
                    );
                });
            });
    }

    fn show_main_screen(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(Frame::default().fill(Color32::BLACK))
            .show(ctx, |_ui| {});
    }

    fn paint_background(&self, ctx: &Context) {
        let rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());

        painter.rect_filled(rect, 0.0, BACKGROUND);
        painter.circle_filled(
            pos2(rect.left() + rect.width() * 0.28, rect.top() + 110.0),
            250.0,
            Color32::from_rgba_unmultiplied(0, 255, 136, 16),
        );
        painter.circle_filled(
            pos2(rect.right() - 120.0, rect.bottom() + 30.0),
            220.0,
            Color32::from_rgba_unmultiplied(0, 255, 136, 8),
        );
    }

    fn show_brand_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.draw_brand_mark(ui);
            ui.add_space(4.0);
            ui.label(
                RichText::new("Epidote")
                    .size(32.0)
                    .strong()
                    .color(PRIMARY),
            );
        });
    }

    fn draw_brand_mark(&self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let stroke = Stroke::new(1.5, PRIMARY);

        let left = pos2(rect.left() + 8.0, rect.center().y);
        let top = pos2(rect.center().x, rect.top() + 8.0);
        let right = pos2(rect.right() - 8.0, rect.center().y);
        let bottom = pos2(rect.center().x, rect.bottom() - 8.0);

        painter.line_segment([left, top], stroke);
        painter.line_segment([top, right], stroke);
        painter.line_segment([left, bottom], stroke);
        painter.line_segment([bottom, right], stroke);

        for point in [left, top, right, bottom] {
            painter.circle_filled(point, 3.5, PRIMARY);
        }
    }

    fn show_login_card(&mut self, ui: &mut Ui) {
        Frame::default()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, OUTLINE))
            .corner_radius(4)
            .inner_margin(Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Auth_v2.4")
                                .size(11.0)
                                .monospace()
                                .color(MUTED_TEXT),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new("Secure Access")
                        .size(26.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new("Authenticate to access the technical intelligence archive.")
                        .size(14.0)
                        .color(MUTED_TEXT),
                );

                ui.add_space(18.0);

                let mut open_main = false;

                ui.scope(|ui| {
                    let visuals = &mut ui.style_mut().visuals;
                    visuals.widgets.inactive.bg_fill = SURFACE_HIGH;
                    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                    visuals.widgets.inactive.fg_stroke.color = TEXT;
                    visuals.widgets.hovered.bg_fill = SURFACE_HIGHEST;
                    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, OUTLINE);
                    visuals.widgets.hovered.fg_stroke.color = TEXT;
                    visuals.widgets.active.bg_fill = SURFACE_HIGHEST;
                    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY_SOFT);
                    visuals.widgets.active.fg_stroke.color = TEXT;
                    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 255, 136, 80);

                    ui.label(
                        RichText::new("IDENTIFIER (EMAIL)")
                            .size(11.0)
                            .strong()
                            .color(MUTED_TEXT),
                    );

                    ui.add_sized(
                        [ui.available_width(), 40.0],
                        TextEdit::singleline(&mut self.email).hint_text("operative@epidote.io"),
                    );

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("ACCESS KEY")
                                .size(11.0)
                                .strong()
                                .color(MUTED_TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("Recover Key")
                                    .size(12.0)
                                    .color(PRIMARY_SOFT),
                            );
                        });
                    });

                    let password_response = ui.add_sized(
                        [ui.available_width(), 40.0],
                        TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("................"),
                    );

                    if password_response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter))
                    {
                        open_main = true;
                    }
                });

                ui.add_space(16.0);

                let primary_button = Button::new(
                    RichText::new("Initialize Session")
                        .size(14.0)
                        .strong()
                        .color(BUTTON_TEXT),
                )
                .fill(PRIMARY)
                .stroke(Stroke::NONE)
                .corner_radius(4)
                .min_size(Vec2::new(ui.available_width(), 46.0));

                if ui.add(primary_button).clicked() {
                    open_main = true;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("SSO Protocol").size(12.0).color(MUTED_TEXT));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let secondary_button = Button::new(
                            RichText::new("Authenticate").size(12.0).color(TEXT),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, OUTLINE))
                        .corner_radius(4);

                        if ui.add(secondary_button).clicked() {
                            open_main = true;
                        }
                    });
                });

                if open_main {
                    self.screen = Screen::Main;
                }
            });
    }

    fn show_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Documentation").size(12.0).color(MUTED_TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("System Status").size(12.0).color(MUTED_TEXT));
            });
        });
    }
}
