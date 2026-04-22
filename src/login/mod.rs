use eframe::egui::{
    self, pos2, vec2, Align, Button, CentralPanel, Color32, Context, FontId, Frame, Key, Layout,
    Margin, Rect, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
};

use crate::{fonts, icons, theme};

/// Renders the login screen. Returns `true` when the user initializes a session
/// (primary button clicked, Enter pressed in the password field, or SSO path).
pub fn show(ctx: &Context, email: &mut String, password: &mut String) -> bool {
    paint_background(ctx);

    let mut open_main = false;

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
                        show_brand_header(ui);
                        ui.add_space(28.0);
                        if show_login_card(ui, email, password) {
                            open_main = true;
                        }
                        ui.add_space(18.0);
                        show_footer(ui);
                    },
                );
            });
        });

    open_main
}

fn paint_background(ctx: &Context) {
    let rect = ctx.screen_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    painter.rect_filled(rect, 0.0, theme::BACKGROUND);

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

fn show_brand_header(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icons::HUB,
            fonts::icon(32.0),
            theme::PRIMARY,
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Epidote")
                .font(fonts::display(30.0))
                .strong()
                .color(theme::PRIMARY),
        );
    });
}

fn show_login_card(ui: &mut Ui, email: &mut String, password: &mut String) -> bool {
    let mut open_main = false;

    Frame::default()
        .fill(theme::SURFACE_CONTAINER)
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .corner_radius(2)
        .inner_margin(Margin::same(32))
        .show(ui, |ui| {
            // Floating "AUTH_V2.4" tag pinned to the top-right of the card.
            let header_rect = ui.available_rect_before_wrap();
            let tag_pos = pos2(header_rect.right(), header_rect.top());
            ui.painter().text(
                tag_pos,
                egui::Align2::RIGHT_TOP,
                "AUTH_V2.4",
                FontId::monospace(10.0),
                theme::DIM_TEXT,
            );

            ui.label(
                RichText::new("Secure Access")
                    .font(fonts::display(24.0))
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(-4.0);
            ui.label(
                RichText::new("Authenticate to access the technical intelligence archive.")
                    .size(13.0)
                    .color(theme::DIM_TEXT),
            );

            ui.add_space(20.0);

            ui.scope(|ui| {
                let v = &mut ui.style_mut().visuals;
                v.widgets.inactive.bg_fill = theme::SURFACE_HIGH;
                v.widgets.inactive.weak_bg_fill = theme::SURFACE_HIGH;
                v.widgets.inactive.bg_stroke = Stroke::NONE;
                v.widgets.inactive.fg_stroke.color = theme::TEXT;
                v.widgets.hovered.bg_fill = theme::SURFACE_HIGH;
                v.widgets.hovered.weak_bg_fill = theme::SURFACE_HIGH;
                v.widgets.hovered.bg_stroke = Stroke::NONE;
                v.widgets.hovered.fg_stroke.color = theme::TEXT;
                v.widgets.active.bg_fill = theme::SURFACE_HIGHEST;
                v.widgets.active.weak_bg_fill = theme::SURFACE_HIGHEST;
                v.widgets.active.bg_stroke = Stroke::new(1.0, theme::PRIMARY_SOFT);
                v.widgets.active.fg_stroke.color = theme::TEXT;
                v.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 255, 136, 80);

                ui.label(
                    RichText::new("IDENTIFIER (EMAIL)")
                        .size(10.5)
                        .strong()
                        .color(theme::DIM_TEXT),
                );
                ui.add_space(2.0);

                ui.add_sized(
                    [ui.available_width(), 44.0],
                    TextEdit::singleline(email)
                        .hint_text("operative@epidote.io")
                        .margin(Margin::symmetric(12, 12)),
                );

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("ACCESS KEY")
                            .size(10.5)
                            .strong()
                            .color(theme::DIM_TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Recover Key")
                                .size(11.5)
                                .color(theme::PRIMARY),
                        );
                    });
                });
                ui.add_space(2.0);

                let password_response = ui.add_sized(
                    [ui.available_width(), 44.0],
                    TextEdit::singleline(password)
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
                RichText::new(format!("Initialize Session  {}", icons::ARROW_FORWARD))
                    .size(13.5)
                    .strong()
                    .color(theme::BUTTON_TEXT),
            )
            .fill(theme::PRIMARY)
            .stroke(Stroke::NONE)
            .corner_radius(2)
            .min_size(Vec2::new(ui.available_width(), 48.0));

            if ui.add(primary_button).clicked() {
                open_main = true;
            }

            ui.add_space(14.0);
            draw_hairline(ui);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SSO Protocol")
                        .size(12.0)
                        .color(theme::DIM_TEXT),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if sso_button(ui).clicked() {
                        open_main = true;
                    }
                });
            });
        });

    open_main
}

fn draw_hairline(ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = rect.top();
    let line = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), 1.0));
    ui.painter().rect_filled(line, 0.0, theme::OUTLINE_VARIANT);
    ui.advance_cursor_after_rect(line);
}

fn sso_button(ui: &mut Ui) -> egui::Response {
    let desired = vec2(130.0, 30.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let painter = ui.painter_at(rect);

    let (fill, stroke_color) = if response.hovered() {
        (theme::SURFACE_HIGHEST, theme::OUTLINE)
    } else {
        (Color32::TRANSPARENT, theme::OUTLINE_VARIANT)
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
        icons::VPN_KEY,
        fonts::icon(16.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.left() + 26.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "Authenticate",
        FontId::proportional(12.0),
        theme::TEXT,
    );

    response
}

fn show_footer(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Documentation").size(11.5).color(theme::DIM_TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new("System Status").size(11.5).color(theme::DIM_TEXT));
        });
    });
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
