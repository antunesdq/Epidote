use std::sync::Arc;

use eframe::egui::{self, FontFamily, FontId};

/// Registers the three bundled typefaces and exposes them via:
/// - `FontFamily::Proportional` — Inter (primary) with Material Symbols as a glyph
///   fallback so PUA codepoints embedded in regular labels still render.
/// - `FontFamily::Name("display")` — Space Grotesk for brand/headline text.
/// - `FontFamily::Name("icons")` — Material Symbols, addressed by `crate::icons`.
pub fn setup(ctx: &egui::Context) {
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

    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
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

pub fn icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("icons".into()))
}

pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("display".into()))
}
