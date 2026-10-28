use eframe::egui::{Color32, Context, Stroke, Visuals};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn apply(self, ctx: &Context) {
        // Match the GCS's main text and controller-panel palette.
        let (mut visuals, background, foreground, control, hover) = match self {
            Self::Light => (Visuals::light(), 0xdc, 0x1b, 0xb1, 0x9f),
            Self::Dark => (Visuals::dark(), 0x12, 0xff, 0x3d, 0x52),
        };
        let foreground = Color32::from_gray(foreground);
        visuals.override_text_color = Some(foreground);
        visuals.panel_fill = Color32::from_gray(background);
        visuals.window_fill = Color32::from_gray(match self {
            Self::Light => 0xc7,
            Self::Dark => 0x24,
        });
        visuals.extreme_bg_color = visuals.panel_fill;
        visuals.faint_bg_color = Color32::from_gray(control);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, foreground);
        visuals.widgets.inactive.bg_fill = Color32::from_gray(control);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(control);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, foreground);
        visuals.widgets.hovered.bg_fill = Color32::from_gray(hover);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(hover);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, foreground);
        visuals.selection.bg_fill = Color32::from_rgb(0x1a, 0xc3, 0xe5);
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_gray(0x1b));
        ctx.set_visuals(visuals);
    }
}
