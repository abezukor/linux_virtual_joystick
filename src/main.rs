use eframe::egui::{self, NumExt, Rect, Sense, Vec2};

mod artwork;
mod browser;
mod controller;
mod function_keys;
mod gamepads;
mod hover;
mod theme;

use controller::ControllerKind;
use hover::controller_hovered;

fn main() -> anyhow::Result<()> {
    let kind = ControllerKind::Xbox;
    let (axes, buttons) = controller::build_uninput(kind)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 620.0])
            .with_min_inner_size([940.0, 540.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Virtual Controller",
        options,
        Box::new(move |cc| {
            theme::Theme::Dark.apply(&cc.egui_ctx);
            Box::new(UI {
                theme: theme::Theme::Dark,
                menu_open: false,
                github_logo: artwork::github_logo(&cc.egui_ctx),
                resize_hover: hover::ResizeHover::default(),
                artwork: artwork::Artwork::new(&cc.egui_ctx, false),
                kind,
                xbox_artwork: Some(artwork::XboxArtwork::new(&cc.egui_ctx)),
                switch_error: None,
                axes,
                spring_axes: [false; 8],
                buttons,
                held_button: None,
                function_keys: None,
                functions_pressed: [false; 10],
                held_function: None,
                dpad_directions: [false; 4],
            })
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

struct UI {
    theme: theme::Theme,
    menu_open: bool,
    github_logo: egui::TextureHandle,
    resize_hover: hover::ResizeHover,
    kind: ControllerKind,
    xbox_artwork: Option<artwork::XboxArtwork>,
    switch_error: Option<String>,
    axes: Box<[controller::AnalogAxis]>,
    spring_axes: [bool; 8],
    buttons: Box<[controller::Button]>,
    artwork: artwork::Artwork,
    held_button: Option<usize>,
    function_keys: Option<function_keys::FunctionKeys>,
    functions_pressed: [bool; 10],
    held_function: Option<usize>,
    dpad_directions: [bool; 4],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonId {
    Joystick(usize),
    Function(usize),
}

struct Picture {
    surface: Rect,
    painter: egui::Painter,
    hovered_button: Option<ButtonId>,
    hovered_axes: [bool; 5],
}

// Use egui's own checkbox rendering, including its text color and border, for
// linked hover. This changes appearance without synthesizing clicks or input.
fn linked_checkbox(
    ui: &mut egui::Ui,
    value: &mut bool,
    text: &str,
    hovered: bool,
) -> egui::Response {
    ui.scope(|ui| {
        if hovered {
            ui.visuals_mut().widgets.inactive = ui.visuals().widgets.hovered;
        }
        ui.checkbox(value, text)
    })
    .inner
}

// Match egui's native checkbox interaction and visuals, with the icon at the
// right edge and its label on the left for the mirrored controls column.
fn mirrored_checkbox(
    ui: &mut egui::Ui,
    value: &mut bool,
    text: &str,
    linked_hover: bool,
) -> egui::Response {
    let galley =
        egui::WidgetText::from(text).into_galley(ui, None, f32::INFINITY, egui::TextStyle::Button);
    let spacing = ui.spacing();
    let icon_width = spacing.icon_width;
    let icon_spacing = spacing.icon_spacing;
    let size = Vec2::new(icon_width + icon_spacing + galley.size().x, galley.size().y)
        .at_least(spacing.interact_size);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *value, text));
    let mut visual_response = response.clone();
    visual_response.hovered =
        controller_hovered(&response) || (hover::enabled(ui.ctx()) && linked_hover);
    let visuals = *ui.style().interact(&visual_response);
    let (small, big) = ui.spacing().icon_rectangles(rect);
    let offset = Vec2::new(rect.width() - icon_width, 0.0);
    let small = small.translate(offset);
    let big = big.translate(offset);
    ui.painter().add(egui::epaint::RectShape::new(
        big.expand(visuals.expansion),
        visuals.rounding,
        visuals.bg_fill,
        visuals.bg_stroke,
    ));
    if *value {
        ui.painter().add(egui::Shape::line(
            vec![
                egui::pos2(small.left(), small.center().y),
                egui::pos2(small.center().x, small.bottom()),
                egui::pos2(small.right(), small.top()),
            ],
            visuals.fg_stroke,
        ));
    }
    let position = egui::pos2(
        rect.right() - icon_width - icon_spacing - galley.size().x,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(position, galley, visuals.text_color());
    response
}

#[derive(Clone, Copy)]
enum SideControl {
    Joystick(usize),
    Button(usize),
    Dpad,
}

#[derive(Default)]
struct ControlsHover {
    button: Option<ButtonId>,
    axes: [bool; 5],
}

// Physical top-to-bottom order, with rear buttons at the bottom of each side.
const LEFT_CONTROLS: [SideControl; 5] = [
    SideControl::Button(8),
    SideControl::Button(6),
    SideControl::Joystick(0),
    SideControl::Dpad,
    SideControl::Button(3),
];
const RIGHT_CONTROLS: [SideControl; 6] = [
    SideControl::Button(9),
    SideControl::Button(7),
    SideControl::Joystick(1),
    SideControl::Button(0),
    SideControl::Button(1),
    SideControl::Button(4),
];
// Each slider's corresponding stick, D-pad, or rotating shoulder in SPECIAL_ART.
const AXIS_CONTROLS: [usize; 8] = [0, 0, 1, 1, 2, 2, 3, 4];
const FUNCTION_ART: [[f32; 4]; 10] = [
    [105.0, 178.0, 35.0, 30.0],
    [106.0, 216.0, 35.0, 30.0],
    [107.0, 254.0, 35.0, 30.0],
    [108.0, 292.0, 34.0, 30.0],
    [591.0, 139.0, 35.0, 30.0],
    [590.0, 178.0, 35.0, 30.0],
    [589.0, 216.0, 35.0, 30.0],
    [588.0, 254.0, 35.0, 30.0],
    [588.0, 292.0, 35.0, 30.0],
    [588.0, 329.0, 35.0, 30.0],
];

// Source rectangles from SrocActive5Controller.qml, in its 730 x 435 design space.
const BUTTON_ART: [(usize, [f32; 4], bool); 8] = [
    (0, [660.0, 228.0, 32.0, 27.0], false),
    (1, [648.0, 269.0, 31.0, 27.0], false),
    (3, [150.0, 283.0, 30.0, 94.0], true),
    (4, [552.0, 283.0, 30.0, 93.0], true),
    (6, [29.0, 127.0, 31.0, 26.0], false),
    (7, [671.0, 127.0, 32.0, 26.0], false),
    (8, [49.0, 75.0, 32.0, 19.0], false),
    (9, [650.0, 75.0, 32.0, 19.0], false),
];
const SPECIAL_ART: [[f32; 4]; 5] = [
    [38.0, 171.0, 49.0, 48.0],  // Left stick
    [646.0, 171.0, 49.0, 48.0], // Right stick
    [41.0, 224.0, 47.0, 45.0],  // D-pad
    [34.0, 48.0, 58.0, 31.0],   // Left rotating shoulder
    [639.0, 48.0, 59.0, 31.0],  // Right rotating shoulder
];

impl UI {
    fn switch_controller(&mut self, ctx: &egui::Context, kind: ControllerKind) {
        match controller::build_uninput(kind) {
            Ok((axes, buttons)) => {
                // Removing the old device also releases its held input in SDL.
                // Drop the keyboard sender to release any held Active5 Fn keys.
                self.function_keys = None;
                self.functions_pressed.fill(false);
                self.dpad_directions.fill(false);
                self.held_button = None;
                self.held_function = None;
                self.axes = axes;
                self.spring_axes.fill(false);
                self.buttons = buttons;
                self.kind = kind;
                self.switch_error = None;
                if kind == ControllerKind::Xbox {
                    self.xbox_artwork = Some(artwork::XboxArtwork::new(ctx));
                } else {
                    self.xbox_artwork = None;
                    if kind != ControllerKind::LinuxVirtualJoystick {
                        self.artwork = artwork::Artwork::new(ctx, kind == ControllerKind::DualGrip);
                    }
                    if kind == ControllerKind::Active5 {
                        self.function_keys = function_keys::FunctionKeys::new().ok();
                    }
                }
            }
            Err(error) => {
                self.switch_error = Some(format!("Could not create {}: {error}", kind.label()))
            }
        }
    }

    fn side_controls(
        &mut self,
        ui: &mut egui::Ui,
        columns: [Rect; 2],
        picture: &Picture,
        controls: &[SideControl],
        hover: &mut ControlsHover,
    ) {
        let [main, adjacent] = columns;
        let mirrored = main.center().x < picture.surface.center().x;
        let layout = egui::Layout::top_down(if mirrored {
            egui::Align::Max
        } else {
            egui::Align::Min
        });
        let source_center_y = |geometry: [f32; 4]| {
            picture.surface.top()
                + (geometry[1] + geometry[3] * 0.5) * picture.surface.width() / 730.0
        };
        let inner_edge = if mirrored {
            adjacent.right()
        } else {
            adjacent.left()
        };
        let trigger_axis = if mirrored { 6 } else { 7 };
        let trigger_size = Self::axis_size();
        let trigger_x = inner_edge
            + if mirrored {
                -trigger_size.x * 0.5
            } else {
                trigger_size.x * 0.5
            };
        let trigger_geometry = SPECIAL_ART[if mirrored { 3 } else { 4 }];
        let trigger_rect = Rect::from_center_size(
            egui::pos2(trigger_x, source_center_y(trigger_geometry)),
            trigger_size,
        );
        let mut child = ui.child_ui(trigger_rect, egui::Layout::top_down(egui::Align::Min));
        self.axis_control(&mut child, picture, trigger_axis, hover);
        for control in controls {
            match *control {
                SideControl::Joystick(index) => {
                    let y = source_center_y(SPECIAL_ART[index]);
                    let x = if mirrored {
                        inner_edge - Self::axis_size().x * 0.5
                    } else {
                        inner_edge + Self::axis_size().x * 0.5
                    };
                    self.stick_controls(ui, picture, index, egui::pos2(x, y), hover);
                }
                SideControl::Dpad => {
                    let slider_bottom = source_center_y(SPECIAL_ART[0]) + 40.0;
                    let rear = BUTTON_ART
                        .iter()
                        .find(|entry| entry.0 == 3)
                        .expect("rear artwork")
                        .1;
                    let rear_top = source_center_y(rear) - 9.0;
                    // Align the rightmost D-pad icon with the left bumper icon.
                    let icon_offset = (18.0 - ui.spacing().icon_width) * 0.5;
                    let rect = Rect::from_center_size(
                        egui::pos2(
                            adjacent.center().x - (44.0 + ui.spacing().icon_width * 0.5) + 31.0,
                            (slider_bottom + rear_top) * 0.5 + icon_offset,
                        ),
                        Vec2::splat(62.0),
                    );
                    let mut child = ui.child_ui(rect, layout);
                    self.dpad_controls(&mut child, picture, hover);
                }
                SideControl::Button(index) => {
                    let geometry = BUTTON_ART
                        .iter()
                        .find(|entry| entry.0 == index)
                        .expect("button artwork")
                        .1;
                    let y = match index {
                        6 | 7 => {
                            let trigger = if index == 6 { 8 } else { 9 };
                            let trigger_geometry = BUTTON_ART
                                .iter()
                                .find(|entry| entry.0 == trigger)
                                .expect("trigger artwork")
                                .1;
                            source_center_y(trigger_geometry) + 42.0
                        }
                        0 | 1 => {
                            let stick_y = source_center_y(SPECIAL_ART[1]);
                            let rear = BUTTON_ART
                                .iter()
                                .find(|entry| entry.0 == 4)
                                .expect("rear artwork")
                                .1;
                            (stick_y + source_center_y(rear)) * 0.5
                                + 8.0
                                + if index == 0 { -11.0 } else { 11.0 }
                        }
                        8 | 9 => source_center_y(geometry) + 20.0,
                        _ => source_center_y(geometry),
                    };
                    // Keep checkbox icons fixed at the inner edge of each side column.
                    let half_icon = ui.spacing().icon_width * 0.5;
                    let (left, right) = if mirrored {
                        (main.left(), adjacent.center().x + half_icon)
                    } else {
                        (adjacent.center().x - half_icon, main.right())
                    };
                    let rect =
                        Rect::from_min_max(egui::pos2(left, y - 9.0), egui::pos2(right, y + 9.0));
                    let mut child = ui.child_ui(rect, layout);
                    let id = ButtonId::Joystick(index);
                    let button = &mut self.buttons[index];
                    let name = button.name();
                    let linked = picture.hovered_button == Some(id);
                    let response = if mirrored {
                        mirrored_checkbox(&mut child, &mut button.new_value, name, linked)
                    } else {
                        linked_checkbox(&mut child, &mut button.new_value, name, linked)
                    };
                    if controller_hovered(&response) {
                        hover.button = Some(id);
                    }
                }
            }
        }
    }

    fn axis_size() -> Vec2 {
        Vec2::new(90.0, 18.0)
    }

    fn stick_controls(
        &mut self,
        ui: &mut egui::Ui,
        picture: &Picture,
        stick: usize,
        center: egui::Pos2,
        hover: &mut ControlsHover,
    ) {
        for (axis, offset) in [(stick * 2, -14.0), (stick * 2 + 1, 14.0)] {
            let rect = Rect::from_center_size(center + Vec2::new(0.0, offset), Self::axis_size());
            let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
            self.axis_control(&mut child, picture, axis, hover);
        }
    }

    fn axis_control(
        &mut self,
        ui: &mut egui::Ui,
        picture: &Picture,
        index: usize,
        hover: &mut ControlsHover,
    ) {
        let axis = &mut self.axes[index];
        let name = axis.name();
        let size = Self::axis_size();
        let length = size.x;
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let mut child = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        child.spacing_mut().slider_width = length;
        if picture.hovered_axes[AXIS_CONTROLS[index]] {
            child.visuals_mut().widgets.inactive = child.visuals().widgets.hovered;
        }
        // The left shoulder slider runs opposite to its raw axis so it follows
        // the mirrored shoulder artwork. Up/down sliders increase from Up to Dn,
        // matching the raw stick Y axis.
        // Xbox LT starts at zero on the right and increases toward the left.
        let inverted = index == 6 && self.kind != ControllerKind::Xbox;
        let reversed_trigger = self.kind == ControllerKind::Xbox && index == 6;
        let mut displayed = if reversed_trigger {
            100 - axis.new_value
        } else if inverted {
            -axis.new_value
        } else {
            axis.new_value
        };
        let minimum = if self.kind == ControllerKind::Xbox && index >= 6 {
            0
        } else {
            -100
        };
        let slider = egui::Slider::new(&mut displayed, minimum..=100).show_value(false);
        let response = child.add(slider);
        response.widget_info(|| egui::WidgetInfo::slider(axis.new_value as f64, name));
        hover.axes[AXIS_CONTROLS[index]] |= controller_hovered(&response);
        if response.changed() {
            self.spring_axes[index] = false;
            axis.new_value = if reversed_trigger {
                100 - displayed
            } else if inverted {
                -displayed
            } else {
                displayed
            };
        }

        // Keep value editors on the outside, away from the controller image.
        let mirrored = rect.center().x < picture.surface.center().x;
        let field_size = Vec2::new(44.0, 18.0);
        let field_x = if mirrored {
            rect.left() - field_size.x - 6.0
        } else {
            rect.right() + 6.0
        };
        let field_y = rect.top();
        let field_rect = Rect::from_min_size(egui::pos2(field_x, field_y), field_size);
        let mut field = ui.child_ui(field_rect, egui::Layout::left_to_right(egui::Align::Center));
        field.spacing_mut().interact_size = field_size;
        let value_response = field.add_sized(
            field_size,
            egui::DragValue::new(&mut axis.new_value)
                .clamp_range(minimum..=100)
                .speed(1.0)
                .fixed_decimals(0),
        );
        if value_response.changed() {
            self.spring_axes[index] = false;
        }
        hover.axes[AXIS_CONTROLS[index]] |= controller_hovered(&value_response);
        if index < 4 {
            let (left, right, y, left_anchor, right_anchor) = if index % 2 == 0 {
                (
                    "L",
                    "R",
                    rect.top() + 2.0,
                    egui::Align2::LEFT_BOTTOM,
                    egui::Align2::RIGHT_BOTTOM,
                )
            } else {
                (
                    "Up",
                    "Dn",
                    rect.bottom() - 2.0,
                    egui::Align2::LEFT_TOP,
                    egui::Align2::RIGHT_TOP,
                )
            };
            let color = ui.visuals().text_color().gamma_multiply(0.5);
            let font = egui::FontId::proportional(12.0);
            ui.painter().text(
                egui::pos2(rect.left(), y),
                left_anchor,
                left,
                font.clone(),
                color,
            );
            ui.painter().text(
                egui::pos2(rect.right(), y),
                right_anchor,
                right,
                font,
                color,
            );
        }
    }

    fn image_axis(&mut self, index: usize, value: i8) {
        self.axes[index].new_value = value;
        self.spring_axes[index] = true;
    }

    fn apply_dpad_directions(&mut self) {
        let [up, down, left, right] = self.dpad_directions;
        self.axes[4].new_value = (i8::from(right) - i8::from(left)) * 100;
        // UI vertical axes increase upward; controller.rs converts this to HAT_Y.
        self.axes[5].new_value = (i8::from(up) - i8::from(down)) * 100;
    }

    fn dpad_controls(&mut self, ui: &mut egui::Ui, picture: &Picture, hover: &mut ControlsHover) {
        let cell = 18.0;
        let gap = 4.0;
        // Empty corners must not intercept neighboring controls.
        let (_, cross) = ui.allocate_space(Vec2::splat(3.0 * cell + 2.0 * gap));
        let mut group_hover = false;
        let center_button = self.kind.dpad_center();
        for (index, col, row) in [(0, 1, 0), (2, 0, 1), (4, 1, 1), (3, 2, 1), (1, 1, 2)] {
            if index == 4 && center_button.is_none() {
                continue;
            }
            let rect = Rect::from_min_size(
                cross.min + Vec2::new(col as f32, row as f32) * (cell + gap),
                Vec2::splat(cell),
            );
            let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
            child.spacing_mut().interact_size = Vec2::splat(cell);
            let linked = picture.hovered_axes[2];
            let mut checked = match index {
                0 => self.axes[5].new_value > 50,
                1 => self.axes[5].new_value < -50,
                2 => self.axes[4].new_value < -50,
                3 => self.axes[4].new_value > 50,
                4 => center_button.is_some_and(|button| self.buttons[button].new_value),
                _ => unreachable!(),
            };
            let response = linked_checkbox(&mut child, &mut checked, "", linked);
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Checkbox,
                    checked,
                    ["Up", "Down", "Left", "Right", "Center"][index],
                )
            });
            group_hover |= controller_hovered(&response);
            if index == 4 && controller_hovered(&response) {
                hover.button = center_button.map(ButtonId::Joystick);
            }
            if response.changed() {
                if checked {
                    if self.kind == ControllerKind::Xbox {
                        self.dpad_directions[index ^ 1] = false;
                    } else {
                        self.dpad_directions.fill(false);
                    }
                    if let Some(button) = center_button {
                        self.buttons[button].new_value = false;
                    }
                }
                if index == 4 {
                    if let Some(button) = center_button {
                        self.buttons[button].new_value = checked;
                    }
                } else {
                    self.dpad_directions[index] = checked;
                }
                self.apply_dpad_directions();
            }
        }
        hover.axes[2] |= group_hover;
    }

    fn function_row(&mut self, ui: &mut egui::Ui, picture: &Picture, hover: &mut ControlsHover) {
        ui.horizontal(|ui| {
            // Equal slots keep the complete F1–F10 row centered at each size.
            let checkbox_width =
                ((ui.available_width() - 9.0 * ui.spacing().item_spacing.x) / 10.0).min(44.0);
            let row_width = 10.0 * checkbox_width + 9.0 * ui.spacing().item_spacing.x;
            ui.add_space(((ui.available_width() - row_width) * 0.5).max(0.0));
            ui.spacing_mut().interact_size.x = checkbox_width;
            for index in 0..10 {
                let id = ButtonId::Function(index);
                let response = linked_checkbox(
                    ui,
                    &mut self.functions_pressed[index],
                    &format!("F{}", index + 1),
                    picture.hovered_button == Some(id),
                );
                if controller_hovered(&response) {
                    hover.button = Some(id);
                }
            }
        });
    }

    fn picture(&mut self, ui: &mut egui::Ui) -> Picture {
        // Trim transparent atlas margins while retaining all source coordinates
        // and enough room for the rotating shoulders at their full travel.
        let _ = ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), Sense::hover());
        let crop_top = 38.0;
        let crop_height = 352.0;
        let scale = (ui.available_width() / 730.0)
            .min((ui.available_height() - 74.0) / crop_height)
            .max(0.1);
        let (image_area, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), crop_height * scale),
            Sense::hover(),
        );
        let visible =
            Rect::from_center_size(image_area.center(), Vec2::new(730.0, crop_height) * scale);
        let surface = Rect::from_min_size(
            visible.min - Vec2::new(0.0, crop_top * scale),
            Vec2::new(730.0, 435.0) * scale,
        );
        let map = |g: [f32; 4]| {
            Rect::from_min_size(
                surface.min + Vec2::new(g[0], g[1]) * scale,
                Vec2::new(g[2], g[3]) * scale,
            )
        };
        let mut hovered_button = None;
        let mut hovered_axes = [false; 5];
        // Drag the drawn sticks, D-pad, or shoulders to generate input directly.
        for (index, geometry) in SPECIAL_ART.iter().enumerate() {
            let rect = map(*geometry);
            let response = ui.interact(
                rect.expand(3.0 * scale),
                ui.id().with(("axis", index)),
                Sense::drag(),
            );
            hovered_axes[index] = controller_hovered(&response);
            if index == 2 && controller_hovered(&response) {
                hovered_button = Some(ButtonId::Joystick(13));
            }
            if response.is_pointer_button_down_on() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let offset = (pointer - rect.center()) / (40.0 * scale);
                    let x = (offset.x.clamp(-1.0, 1.0) * 100.0) as i8;
                    let y = (offset.y.clamp(-1.0, 1.0) * 100.0) as i8;
                    match index {
                        0 => {
                            self.image_axis(0, x);
                            self.image_axis(1, y);
                        }
                        1 => {
                            self.image_axis(2, x);
                            self.image_axis(3, y);
                        }
                        2 => {
                            self.dpad_directions.fill(false);
                            self.image_axis(4, 0);
                            self.image_axis(5, 0);
                            self.buttons[13].new_value = false;
                            // Snap to the dominant direction; never emit diagonals
                            // or a center press together with a direction.
                            if x.abs().max(y.abs()) <= 50 {
                                self.buttons[13].new_value = true;
                                self.held_button = Some(13);
                            } else if x.abs() > y.abs() {
                                self.image_axis(4, x.signum() * 100);
                            } else {
                                self.image_axis(5, -y.signum() * 100);
                            }
                        }
                        3 => self.image_axis(6, -x),
                        4 => self.image_axis(7, x),
                        _ => unreachable!(),
                    }
                }
            }
        }
        for (index, geometry, _) in BUTTON_ART {
            let response = ui.interact(
                map(geometry),
                ui.id().with(("button", index)),
                Sense::click_and_drag(),
            );
            if controller_hovered(&response) {
                hovered_button = Some(ButtonId::Joystick(index));
            }
            if response.is_pointer_button_down_on() {
                self.buttons[index].new_value = true;
                self.held_button = Some(index);
            }
        }
        for (index, geometry) in FUNCTION_ART.iter().enumerate() {
            let response = ui.interact(
                map(*geometry),
                ui.id().with(("function", index)),
                Sense::click_and_drag(),
            );
            if controller_hovered(&response) {
                hovered_button = Some(ButtonId::Function(index));
            }
            if response.is_pointer_button_down_on() {
                self.functions_pressed[index] = true;
                self.held_function = Some(index);
            }
        }
        let picture = Picture {
            surface,
            painter: ui
                .painter()
                .with_clip_rect(ui.clip_rect().intersect(visible)),
            hovered_button,
            hovered_axes,
        };
        picture
    }

    fn paint_picture(
        &self,
        picture: &Picture,
        checkbox_hover: Option<ButtonId>,
        slider_hover: [bool; 5],
    ) {
        let scale = picture.surface.width() / 730.0;
        let painter = &picture.painter;
        let map = |g: [f32; 4]| {
            Rect::from_min_size(
                picture.surface.min + Vec2::new(g[0], g[1]) * scale,
                Vec2::new(g[2], g[3]) * scale,
            )
        };
        let hovered = |id| picture.hovered_button == Some(id) || checkbox_hover == Some(id);
        for (index, geometry, special) in BUTTON_ART {
            self.artwork.button(
                painter,
                map(geometry),
                geometry,
                self.buttons[index].new_value,
                special,
                hovered(ButtonId::Joystick(index)),
            );
        }
        for (index, geometry) in FUNCTION_ART.iter().enumerate() {
            self.artwork.button(
                painter,
                map(*geometry),
                *geometry,
                self.functions_pressed[index],
                false,
                hovered(ButtonId::Function(index)),
            );
        }
        // Draw all controller and rear-button outlines after the fills and blue
        // highlights, including the body lines which pass behind the rear buttons.
        self.artwork.body(painter, picture.surface);
        for (index, geometry, special) in BUTTON_ART {
            if special && (self.buttons[index].new_value || hovered(ButtonId::Joystick(index))) {
                self.artwork.rear_lines(painter, map(geometry), geometry);
            }
        }
        for (index, geometry) in SPECIAL_ART.iter().enumerate() {
            let (x, y, angle, active) = match index {
                0 => (
                    self.axes[0].new_value as f32 / 100.0,
                    self.axes[1].new_value as f32 / 100.0,
                    0.0,
                    false,
                ),
                1 => (
                    self.axes[2].new_value as f32 / 100.0,
                    self.axes[3].new_value as f32 / 100.0,
                    0.0,
                    false,
                ),
                2 => {
                    let x = self.axes[4].new_value as f32 / 100.0;
                    let y = -self.axes[5].new_value as f32 / 100.0;
                    (
                        if x.abs() > 0.5 { x.signum() } else { 0.0 },
                        if y.abs() > 0.5 { y.signum() } else { 0.0 },
                        0.0,
                        self.buttons[13].new_value,
                    )
                }
                3 => (
                    0.0,
                    0.0,
                    -(self.axes[6].new_value as f32) / 100.0 * 20.0,
                    false,
                ),
                4 => (
                    0.0,
                    0.0,
                    self.axes[7].new_value as f32 / 100.0 * 20.0,
                    false,
                ),
                _ => unreachable!(),
            };
            let travel = if index == 2 { 12.0 } else { 14.0 };
            let destination = map(*geometry).translate(Vec2::new(x, y) * travel * scale);
            self.artwork.special(
                painter,
                destination,
                *geometry,
                angle.to_radians(),
                active || x.abs() > 0.02 || y.abs() > 0.02 || angle.abs() > 0.4,
                picture.hovered_axes[index]
                    || slider_hover[index]
                    || (index == 2 && hovered(ButtonId::Joystick(13))),
            );
        }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.resize_hover.begin_frame(ctx);
        if let Some(index) = self.held_button.take() {
            self.buttons[index].new_value = false;
        }
        if let Some(index) = self.held_function.take() {
            self.functions_pressed[index] = false;
        }
        // Only image interactions spring back. Slider/value-editor inputs stay
        // at their last value until changed or replaced by an image interaction.
        for (axis, spring) in self.axes.iter_mut().zip(self.spring_axes.iter_mut()) {
            if std::mem::take(spring) {
                axis.new_value = 0;
            }
        }
        if self.kind != ControllerKind::LinuxVirtualJoystick {
            self.apply_dpad_directions();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if !hover::enabled(ctx) {
                ui.visuals_mut().widgets.hovered = ui.visuals().widgets.inactive;
            }
            let mut selected = self.kind;
            let mut selected_theme = self.theme;
            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    let button = ui.add(
                        egui::Button::new(egui::RichText::new("    Menu").size(24.0))
                            .min_size(Vec2::new(116.0, 36.0)),
                    );
                    let stroke = ui.style().interact(&button).fg_stroke;
                    for offset in [-6.0, 0.0, 6.0] {
                        let start = button.rect.left_center() + Vec2::new(10.0, offset);
                        ui.painter()
                            .line_segment([start, start + Vec2::new(18.0, 0.0)], stroke);
                    }
                    if button.clicked() {
                        self.menu_open = !self.menu_open;
                    }
                    if self.menu_open {
                        let menu = egui::Area::new(egui::Id::new("settings-menu"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(button.rect.left_bottom() + Vec2::new(0.0, 2.0))
                            .show(ctx, |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    for font in ui.style_mut().text_styles.values_mut() {
                                        font.size *= 2.0;
                                    }
                                    let spacing = ui.spacing_mut();
                                    spacing.interact_size *= 2.0;
                                    spacing.button_padding *= 2.0;
                                    spacing.item_spacing *= 2.0;
                                    spacing.icon_width *= 2.0;
                                    spacing.icon_spacing *= 2.0;
                                    ui.set_width(560.0);
                                    let menu_font_size =
                                        egui::TextStyle::Button.resolve(&ctx.style()).size * 2.0;
                                    egui::Grid::new("menu-settings")
                                        .num_columns(2)
                                        .spacing(Vec2::new(24.0, 12.0))
                                        .show(ui, |ui| {
                                            ui.label("Controller");
                                            egui::ComboBox::from_id_source("controller-kind")
                                                .width(360.0)
                                                .selected_text(selected.label())
                                                .show_ui(ui, |ui| {
                                                    ui.spacing_mut().interact_size.y = 36.0;
                                                    ui.spacing_mut().item_spacing.y = 6.0;
                                                    for kind in ControllerKind::ALL {
                                                        ui.selectable_value(
                                                            &mut selected,
                                                            kind,
                                                            egui::RichText::new(kind.label())
                                                                .size(menu_font_size),
                                                        );
                                                    }
                                                });
                                            ui.end_row();
                                            ui.label("Color Theme");
                                            egui::ComboBox::from_id_source("color-theme")
                                                .width(360.0)
                                                .selected_text(selected_theme.label())
                                                .show_ui(ui, |ui| {
                                                    ui.spacing_mut().interact_size.y = 36.0;
                                                    ui.spacing_mut().item_spacing.y = 6.0;
                                                    for appearance in
                                                        [theme::Theme::Light, theme::Theme::Dark]
                                                    {
                                                        ui.selectable_value(
                                                            &mut selected_theme,
                                                            appearance,
                                                            egui::RichText::new(appearance.label())
                                                                .size(menu_font_size),
                                                        );
                                                    }
                                                });
                                            ui.end_row();
                                            ui.label("Version");
                                            ui.allocate_ui_with_layout(
                                                Vec2::new(360.0, 36.0),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    ui.label(env!("CARGO_PKG_VERSION"));
                                                },
                                            );
                                            ui.end_row();
                                            ui.add(
                                                egui::Image::new(&self.github_logo)
                                                    .fit_to_exact_size(Vec2::splat(28.0))
                                                    .tint(ui.visuals().text_color()),
                                            );
                                            let url = "https://github.com/abezukor/linux_virtual_joystick";
                                            let link = ui.hyperlink_to(
                                                egui::RichText::new(url.trim_start_matches("https://"))
                                                    .size(menu_font_size * 0.75)
                                                    .color(ui.visuals().hyperlink_color),
                                                url,
                                            );
                                            if link.clicked() || link.middle_clicked() {
                                                browser::redirect_sudo_link(ctx, url);
                                            }
                                            ui.end_row();
                                        });
                                })
                            });
                        // Let the nested selectors handle clicks in their own
                        // popups before dismissing the menu on an outside click.
                        if !ctx.memory(|memory| memory.any_popup_open())
                            && ctx.input(|input| input.pointer.any_pressed())
                            && !menu.response.contains_pointer()
                            && !button.contains_pointer()
                        {
                            self.menu_open = false;
                        }
                        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                            self.menu_open = false;
                        }
                    }
                });
            });
            if selected_theme != self.theme {
                self.theme = selected_theme;
                self.theme.apply(ctx);
            }
            if selected != self.kind {
                self.switch_controller(ctx, selected);
            }
            if let Some(error) = &self.switch_error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            ui.add_space(4.0);
            if self.kind == ControllerKind::LinuxVirtualJoystick {
                ui.add_space(10.0);
                // The original interface uses egui's standard sliders and
                // checkboxes, with persistent values and no controller image.
                for axis in self.axes.iter_mut() {
                    let name = axis.name();
                    ui.add(egui::Slider::new(&mut axis.new_value, -100..=100).text(name));
                }
                for button in self.buttons.iter_mut() {
                    let name = button.name();
                    ui.add(egui::Checkbox::new(&mut button.new_value, name));
                }
                return;
            }
            if self.kind != ControllerKind::Active5 {
                gamepads::show(self, ui);
                return;
            }
            let (row, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
            let gap = ui.spacing().item_spacing.x;
            let outer_width = 144.0;
            let inner_width = 28.0;
            let side_width = 2.0 * outer_width + 2.0 * inner_width + 4.0 * gap;
            // The image is limited by height in a wide window. Keep the controls
            // beside that rendered image rather than at the window edges.
            let natural_picture_width = ((row.height() - 120.0) / 352.0).max(0.1) * 730.0;
            let row = Rect::from_center_size(
                row.center(),
                Vec2::new(
                    row.width().min(side_width + natural_picture_width),
                    row.height(),
                ),
            );
            let picture_width = row.width() - side_width;
            let column = |x: f32, width: f32| {
                Rect::from_min_size(row.min + Vec2::new(x, 0.0), Vec2::new(width, row.height()))
            };
            let left_outer = column(0.0, outer_width);
            let left_inner = column(outer_width + gap, inner_width);
            let image = column(outer_width + inner_width + 2.0 * gap, picture_width);
            let right_inner = column(image.right() - row.left() + gap, inner_width);
            let right_outer = column(right_inner.right() - row.left() + gap, outer_width);

            // Process picture interactions first so both side columns can use
            // this frame's hover state, regardless of their visual position.
            let mut hover = ControlsHover::default();
            let picture = ui
                .allocate_ui_at_rect(image, |ui| {
                    let picture = self.picture(ui);
                    ui.add_space(18.0);
                    self.function_row(ui, &picture, &mut hover);
                    picture
                })
                .inner;
            self.side_controls(
                ui,
                [left_outer, left_inner],
                &picture,
                &LEFT_CONTROLS,
                &mut hover,
            );
            self.side_controls(
                ui,
                [right_outer, right_inner],
                &picture,
                &RIGHT_CONTROLS,
                &mut hover,
            );
            self.paint_picture(&picture, hover.button, hover.axes);
        });
        for axis in self.axes.iter_mut() {
            axis.new_value();
        }
        for button in self.buttons.iter_mut() {
            button.new_value();
        }
        if let Some(keys) = &mut self.function_keys {
            if let Err(error) = keys.sync(&self.functions_pressed) {
                eprintln!("Could not send function keys: {error}");
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}
