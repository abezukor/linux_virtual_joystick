//! Controller SVG layers copied from the GCS, using the same source rectangles.
use eframe::egui::{self, Color32, ColorImage, Painter, Rect, TextureHandle, Vec2};

pub fn github_logo(ctx: &egui::Context) -> TextureHandle {
    let tree =
        resvg::usvg::Tree::from_data(include_bytes!("assets/github.svg"), &Default::default())
            .expect("embedded GitHub logo");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(64, 64).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(64.0 / 24.0, 64.0 / 24.0),
        &mut pixmap.as_mut(),
    );
    ctx.load_texture(
        "github-logo",
        ColorImage::from_rgba_premultiplied([64, 64], pixmap.data()),
        Default::default(),
    )
}

pub struct Artwork {
    lines: TextureHandle,
    highlights: TextureHandle,
    idle_highlights: TextureHandle,
    special_fill: TextureHandle,
    special_lines: TextureHandle,
    special_highlights: TextureHandle,
    special_idle: TextureHandle,
    center_keys: Option<TextureHandle>,
}

macro_rules! svg {
    ($suffix:literal) => {
        include_bytes!(concat!(
            "assets/active5/ic_controller_SRoC_Active5",
            $suffix,
            ".svg"
        ))
    };
}

fn texture(ctx: &egui::Context, name: &str, bytes: &[u8], grayscale: bool) -> TextureHandle {
    raster_texture(ctx, name, bytes, grayscale, false)
}

fn solid_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> TextureHandle {
    raster_texture(ctx, name, bytes, false, true)
}

fn raster_texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
    grayscale: bool,
    solid: bool,
) -> TextureHandle {
    let source = String::from_utf8_lossy(bytes);
    let source = if grayscale {
        source.replace("#00D4FF", "#FFFFFF")
    } else {
        source.into_owned()
    };
    let tree = resvg::usvg::Tree::from_data(source.as_bytes(), &Default::default())
        .expect("embedded controller SVG");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(1460, 870).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(2.0, 2.0),
        &mut pixmap.as_mut(),
    );
    if solid {
        // Solid fills are alpha masks so even black source artwork can take
        // the panel's light or dark background color.
        for pixel in pixmap.data_mut().chunks_exact_mut(4) {
            pixel[0] = pixel[3];
            pixel[1] = pixel[3];
            pixel[2] = pixel[3];
        }
    }
    ctx.load_texture(
        name,
        ColorImage::from_rgba_premultiplied([1460, 870], pixmap.data()),
        Default::default(),
    )
}

impl Artwork {
    pub fn new(ctx: &egui::Context, dual_grip: bool) -> Self {
        macro_rules! dual {
            ($suffix:literal) => {
                include_bytes!(concat!(
                    "assets/dualgrip/ic_controller_SRoC_DualGrip",
                    $suffix,
                    ".svg"
                )) as &[u8]
            };
        }
        let sources: [&[u8]; 5] = if dual_grip {
            [
                dual!("_lines"),
                dual!("_highlights"),
                dual!("_special_inputs_solid_fill"),
                dual!("_special_inputs_lines"),
                dual!("_special_inputs_highlights"),
            ]
        } else {
            [
                svg!("_lines"),
                svg!("_highlights"),
                svg!("_special_inputs_solid_fill"),
                svg!("_special_inputs_lines"),
                svg!("_special_inputs_highlights"),
            ]
        };
        let prefix = if dual_grip { "dualgrip" } else { "active5" };
        let load = |suffix: &str, source, grayscale| {
            texture(ctx, &format!("{prefix}-{suffix}"), source, grayscale)
        };
        Self {
            lines: load("lines", sources[0], false),
            highlights: load("highlights", sources[1], false),
            idle_highlights: load("idle", sources[1], true),
            special_fill: solid_texture(ctx, &format!("{prefix}-special-fill"), sources[2]),
            special_lines: load("special-lines", sources[3], false),
            special_highlights: load("special-highlights", sources[4], false),
            special_idle: load("special-idle", sources[4], true),
            // The DualGrip atlas omits fills for its disabled center keys.
            // Reuse the GCS Active5 Fn-key sprite for these virtual HID keys.
            center_keys: dual_grip.then(|| load("center-key", svg!("_highlights"), false)),
        }
    }

    fn layer(
        painter: &Painter,
        texture: &TextureHandle,
        target: Rect,
        source: [f32; 4],
        tint: Color32,
        angle: f32,
    ) {
        let uv = Rect::from_min_size(
            egui::pos2(source[0] / 730.0, source[1] / 435.0),
            Vec2::new(source[2] / 730.0, source[3] / 435.0),
        );
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.add_rect_with_uv(target, uv, tint);
        mesh.rotate(egui::emath::Rot2::from_angle(angle), target.center());
        painter.add(egui::Shape::mesh(mesh));
    }

    pub fn body(&self, painter: &Painter, target: Rect) {
        Self::layer(
            painter,
            &self.lines,
            target,
            [0.0, 0.0, 730.0, 435.0],
            painter.ctx().style().visuals.text_color(),
            0.0,
        );
    }

    pub fn button(
        &self,
        painter: &Painter,
        target: Rect,
        source: [f32; 4],
        active: bool,
        special: bool,
        hovered: bool,
    ) {
        if special && !active && !hovered {
            return;
        }
        let (idle, highlight) = if special {
            (&self.special_idle, &self.special_highlights)
        } else {
            (&self.idle_highlights, &self.highlights)
        };
        let white = painter
            .ctx()
            .style()
            .visuals
            .text_color()
            .gamma_multiply(if hovered { 1.0 } else { 0.3 });
        Self::layer(painter, idle, target, source, white, 0.0);
        if active {
            Self::layer(painter, highlight, target, source, Color32::WHITE, 0.0);
        }
    }

    pub fn rear_lines(&self, painter: &Painter, target: Rect, source: [f32; 4]) {
        Self::layer(
            painter,
            &self.special_lines,
            target,
            source,
            painter.ctx().style().visuals.text_color(),
            0.0,
        );
    }

    pub fn center_key(&self, painter: &Painter, target: Rect, active: bool) {
        if active {
            let highlight = self.center_keys.as_ref().expect("DualGrip center artwork");
            Self::layer(
                painter,
                highlight,
                target,
                [105.0, 178.0, 35.0, 30.0],
                Color32::WHITE,
                0.0,
            );
        }
    }

    pub fn special(
        &self,
        painter: &Painter,
        target: Rect,
        source: [f32; 4],
        angle: f32,
        active: bool,
        hovered: bool,
    ) {
        Self::layer(
            painter,
            &self.special_fill,
            target,
            source,
            painter.ctx().style().visuals.panel_fill,
            angle,
        );
        let white = painter
            .ctx()
            .style()
            .visuals
            .text_color()
            .gamma_multiply(if hovered { 1.0 } else { 0.3 });
        Self::layer(painter, &self.special_idle, target, source, white, angle);
        if active {
            Self::layer(
                painter,
                &self.special_highlights,
                target,
                source,
                Color32::WHITE,
                angle,
            );
        }
        Self::layer(
            painter,
            &self.special_lines,
            target,
            source,
            painter.ctx().style().visuals.text_color(),
            angle,
        );
    }
}

pub struct XboxArtwork {
    lines: TextureHandle,
    highlights: [TextureHandle; 2],
    idle: [TextureHandle; 2],
    cap_lines: TextureHandle,
    cap_fill: TextureHandle,
    paddle_lines: [TextureHandle; 2],
}

impl XboxArtwork {
    pub fn new(ctx: &egui::Context) -> Self {
        macro_rules! xbox {
            ($suffix:literal) => {
                include_bytes!(concat!(
                    "assets/xbox/ic_controller_xbox_series",
                    $suffix,
                    ".svg"
                ))
            };
        }
        Self {
            lines: texture(ctx, "xbox-lines", xbox!("_lines"), false),
            highlights: [
                texture(ctx, "xbox-high1", xbox!("_highlights1"), false),
                texture(ctx, "xbox-high2", xbox!("_highlights2"), false),
            ],
            idle: [
                texture(ctx, "xbox-idle1", xbox!("_highlights1"), true),
                texture(ctx, "xbox-idle2", xbox!("_highlights2"), true),
            ],
            cap_lines: texture(ctx, "xbox-cap-lines", xbox!("_stick_cap_lines"), false),
            cap_fill: solid_texture(ctx, "xbox-cap-fill", xbox!("_stick_cap_solid_fill")),
            paddle_lines: [
                texture(
                    ctx,
                    "xbox-upper-paddles",
                    xbox!("_back_paddles_upper_lines"),
                    false,
                ),
                texture(
                    ctx,
                    "xbox-lower-paddles",
                    xbox!("_back_paddles_lower_lines"),
                    false,
                ),
            ],
        }
    }

    pub fn body(&self, painter: &Painter, rect: Rect) {
        Artwork::layer(
            painter,
            &self.lines,
            rect,
            [0.0, 0.0, 730.0, 435.0],
            painter.ctx().style().visuals.text_color(),
            0.0,
        );
    }

    pub fn button(
        &self,
        painter: &Painter,
        rect: Rect,
        source: [f32; 4],
        sheet: usize,
        state: (bool, bool),
    ) {
        let (active, hovered) = state;
        Artwork::layer(
            painter,
            &self.idle[sheet],
            rect,
            source,
            painter
                .ctx()
                .style()
                .visuals
                .text_color()
                .gamma_multiply(if hovered { 1.0 } else { 0.3 }),
            0.0,
        );
        if active {
            Artwork::layer(
                painter,
                &self.highlights[sheet],
                rect,
                source,
                Color32::WHITE,
                0.0,
            );
        }
    }

    pub fn trigger(
        &self,
        painter: &Painter,
        rect: Rect,
        source: [f32; 4],
        travel: f32,
        left: bool,
        hovered: bool,
    ) {
        self.button(painter, rect, source, 0, (false, hovered));
        let travel = travel.clamp(0.0, 1.0);
        if travel > 0.0 {
            let mut fill = rect;
            if left {
                fill.min.x = rect.right() - rect.width() * travel;
            } else {
                fill.max.x = rect.left() + rect.width() * travel;
            }
            let clipped = painter.with_clip_rect(painter.clip_rect().intersect(fill));
            Artwork::layer(
                &clipped,
                &self.highlights[0],
                rect,
                source,
                Color32::from_white_alpha(if travel == 1.0 { 255 } else { 128 }),
                0.0,
            );
        }
    }

    pub fn cap(&self, painter: &Painter, rect: Rect, source: [f32; 4], state: (bool, bool)) {
        Artwork::layer(
            painter,
            &self.cap_fill,
            rect,
            source,
            painter.ctx().style().visuals.panel_fill,
            0.0,
        );
        self.button(painter, rect, source, 1, state);
        Artwork::layer(
            painter,
            &self.cap_lines,
            rect,
            source,
            painter.ctx().style().visuals.text_color(),
            0.0,
        );
    }

    pub fn paddle_lines(&self, painter: &Painter, rect: Rect, source: [f32; 4], lower: bool) {
        Artwork::layer(
            painter,
            &self.paddle_lines[usize::from(lower)],
            rect,
            source,
            painter.ctx().style().visuals.text_color(),
            0.0,
        );
    }
}
