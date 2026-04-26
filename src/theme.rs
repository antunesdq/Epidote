use eframe::egui::Color32;

pub const BACKGROUND: Color32 = Color32::from_rgb(0x13, 0x13, 0x13);
pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(0x1b, 0x1b, 0x1c);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const SURFACE_HIGHEST: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
pub const TEXT: Color32 = Color32::from_rgb(0xe5, 0xe2, 0xe1);
pub const DIM_TEXT: Color32 = Color32::from_rgb(0x5b, 0x64, 0x61);
pub const PRIMARY: Color32 = Color32::from_rgb(0x00, 0xff, 0x88);
pub const PRIMARY_SOFT: Color32 = Color32::from_rgb(0x60, 0xff, 0x99);
pub const BUTTON_TEXT: Color32 = Color32::from_rgb(0x00, 0x21, 0x0c);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0x2a, 0x33, 0x2c);
pub const OUTLINE: Color32 = Color32::from_rgb(0x3b, 0x4b, 0x3d);

/// Softer than `DIM_TEXT` — used for body-secondary text where `DIM_TEXT`
/// (which is the muted/quietest tier) feels too far away from `TEXT`.
pub const SOFT_TEXT: Color32 = Color32::from_rgb(0x8a, 0x8f, 0x8b);

pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xce, 0xbd, 0xff);
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(0xff, 0xd5, 0x8a);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0xb4, 0xab);
