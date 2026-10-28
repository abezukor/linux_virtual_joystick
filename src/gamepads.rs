//! DualGrip and Xbox controls, using the GCS's original SVG atlas coordinates.
use crate::{
    controller::ControllerKind, controller_hovered, linked_checkbox, mirrored_checkbox, ButtonId,
    ControlsHover, Picture, UI,
};
use eframe::egui::{self, Rect, Sense, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Input {
    Button(usize),
    Trigger(usize),
    Direction(usize),
}

#[derive(Clone, Copy)]
struct Control {
    input: Input,
    geometry: [f32; 4],
    sheet: usize,
    rear: bool,
}

fn control(input: Input, geometry: [f32; 4], sheet: usize, rear: bool) -> Control {
    Control {
        input,
        geometry,
        sheet,
        rear,
    }
}

fn controls(kind: ControllerKind) -> Vec<Control> {
    use Input::{Button as B, Direction as D, Trigger as T};
    if kind == ControllerKind::DualGrip {
        vec![
            control(B(8), [196.078, 93.612, 53.363, 28.580], 0, false),
            control(B(9), [481.216, 93.612, 53.290, 28.580], 0, false),
            control(B(6), [149.577, 134.806, 54.531, 45.588], 0, false),
            control(B(7), [526.768, 134.806, 54.531, 45.588], 0, false),
            control(B(0), [518.811, 184.397, 54.531, 45.588], 0, false),
            control(B(1), [483.187, 227.592, 54.531, 45.588], 0, false),
            control(B(3), [196.0, 251.0, 64.0, 121.0], 0, true),
            control(B(4), [470.0, 252.0, 67.0, 122.0], 0, true),
            control(B(25), [320.032, 177.219, 35.186, 30.363], 0, false),
            control(B(26), [375.220, 177.219, 35.186, 30.363], 0, false),
            control(B(21), [265.574, 222.807, 35.259, 30.407], 0, false),
            control(B(22), [320.032, 222.807, 35.186, 30.407], 0, false),
            control(B(23), [375.220, 222.807, 35.186, 30.407], 0, false),
            control(B(24), [429.605, 222.807, 35.186, 30.407], 0, false),
        ]
    } else {
        vec![
            control(T(0), [167.0, 42.0, 82.0, 81.0], 0, false),
            control(T(1), [482.0, 42.0, 81.0, 81.0], 0, false),
            control(B(9), [168.0, 89.0, 117.0, 43.0], 1, false),
            control(B(10), [445.0, 89.0, 117.0, 43.0], 1, false),
            control(D(0), [276.0, 261.0, 31.0, 22.0], 0, false),
            control(D(1), [276.0, 306.0, 31.0, 26.0], 0, false),
            control(D(2), [245.0, 282.0, 33.0, 27.0], 1, false),
            control(D(3), [304.0, 282.0, 33.0, 28.0], 1, false),
            control(B(4), [310.0, 204.0, 25.0, 23.0], 1, false),
            control(B(5), [394.0, 204.0, 25.0, 23.0], 1, false),
            control(B(6), [346.0, 141.0, 39.0, 38.0], 1, false),
            control(B(0), [500.0, 230.0, 42.0, 36.0], 0, false),
            control(B(1), [541.0, 190.0, 43.0, 37.0], 1, false),
            control(B(2), [459.0, 196.0, 41.0, 38.0], 1, false),
            control(B(3), [500.0, 159.0, 42.0, 37.0], 0, false),
            control(B(11), [156.0, 301.0, 65.0, 36.0], 0, true),
            control(B(12), [154.0, 321.0, 67.0, 50.0], 1, true),
            control(B(13), [509.0, 301.0, 63.0, 36.0], 0, true),
            control(B(14), [508.0, 321.0, 67.0, 50.0], 1, true),
        ]
    }
}

const DUAL_SPECIAL: [[f32; 4]; 5] = [
    [232.140, 158.470, 63.510, 67.947],
    [435.007, 158.470, 63.510, 67.947],
    [156.804, 187.268, 55.115, 56.681],
    [177.609, 54.810, 69.204, 26.578],
    [483.625, 54.810, 69.204, 26.578],
];
const XBOX_STICKS: [[f32; 4]; 2] = [[187.0, 201.0, 62.0, 54.0], [409.0, 282.0, 60.0, 52.0]];

fn active(app: &UI, input: Input) -> bool {
    match input {
        Input::Button(index) => app.buttons[index].new_value,
        Input::Trigger(index) => app.axes[index + 6].new_value == 100,
        Input::Direction(index) => match index {
            0 => app.axes[5].new_value > 50,
            1 => app.axes[5].new_value < -50,
            2 => app.axes[4].new_value < -50,
            _ => app.axes[4].new_value > 50,
        },
    }
}

fn hovered(input: Input, picture: &Picture, hover: &ControlsHover) -> bool {
    match input {
        Input::Button(index) => {
            picture.hovered_button == Some(ButtonId::Joystick(index))
                || hover.button == Some(ButtonId::Joystick(index))
        }
        Input::Trigger(index) => picture.hovered_axes[index + 3] || hover.axes[index + 3],
        Input::Direction(_) => picture.hovered_axes[2] || hover.axes[2],
    }
}

fn press_direction(app: &mut UI, direction: Option<usize>) {
    app.dpad_directions.fill(false);
    if let Some(center) = app.kind.dpad_center() {
        app.buttons[center].new_value = false;
    }
    app.image_axis(4, 0);
    app.image_axis(5, 0);
    match direction {
        Some(0) => app.image_axis(5, 100),
        Some(1) => app.image_axis(5, -100),
        Some(2) => app.image_axis(4, -100),
        Some(3) => app.image_axis(4, 100),
        _ => {
            if let Some(center) = app.kind.dpad_center() {
                app.buttons[center].new_value = true;
                app.held_button = Some(center);
            }
        }
    }
}

fn sidebar_button(
    app: &mut UI,
    ui: &mut egui::Ui,
    index: usize,
    picture: &Picture,
    hover: &mut ControlsHover,
) {
    let linked = picture.hovered_button == Some(ButtonId::Joystick(index));
    let button = &mut app.buttons[index];
    let name = button.name();
    let response = if ui.layout().main_dir() == egui::Direction::RightToLeft {
        mirrored_checkbox(ui, &mut button.new_value, name, linked)
    } else {
        linked_checkbox(ui, &mut button.new_value, name, linked)
    };
    if controller_hovered(&response) {
        hover.button = Some(ButtonId::Joystick(index));
    }
}

fn dual_sidebar_button(
    app: &mut UI,
    ui: &mut egui::Ui,
    side: usize,
    index: usize,
    picture: &Picture,
    hover: &mut ControlsHover,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
    let layout = if side == 0 {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    let mut child = ui.child_ui(rect, layout);
    sidebar_button(app, &mut child, index, picture, hover);
}

fn sidebar(
    app: &mut UI,
    ui: &mut egui::Ui,
    side: usize,
    picture: &Picture,
    hover: &mut ControlsHover,
) {
    if app.kind == ControllerKind::Xbox {
        xbox_sidebar(app, ui, side, picture, hover);
        return;
    }
    let trigger = controls(ControllerKind::DualGrip)[side].geometry;
    let trigger_y =
        picture.surface.top() + (trigger[1] + trigger[3] * 0.5) * picture.surface.width() / 730.0;
    ui.add_space((trigger_y - 9.0 - ui.cursor().top()).max(0.0));
    let top: &[usize] = if side == 0 { &[8, 6] } else { &[9, 7] };
    for &index in top {
        dual_sidebar_button(app, ui, side, index, picture, hover);
    }
    ui.add_space(16.0);
    let geometry = DUAL_SPECIAL[side];
    let stick_y =
        picture.surface.top() + (geometry[1] + geometry[3] * 0.5) * picture.surface.width() / 730.0;
    ui.add_space((stick_y - 40.0 - ui.cursor().top()).max(0.0));
    let (stick_row, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 80.0), Sense::hover());
    let center_x = if side == 0 {
        stick_row.right() - UI::axis_size().x * 0.5
    } else {
        stick_row.left() + UI::axis_size().x * 0.5
    };
    app.stick_controls(
        ui,
        picture,
        side,
        egui::pos2(center_x, stick_row.center().y),
        hover,
    );
    ui.add_space(16.0);
    if side == 0 {
        let (row, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 62.0), Sense::hover());
        let rect = Rect::from_min_size(
            egui::pos2(row.right() - 44.0 - ui.spacing().icon_width, row.top()),
            Vec2::splat(62.0),
        );
        let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
        app.dpad_controls(&mut child, picture, hover);
    } else if app.kind == ControllerKind::DualGrip {
        for index in [0, 1] {
            dual_sidebar_button(app, ui, side, index, picture, hover);
        }
    }
    let rear_button = if side == 0 { 3 } else { 4 };
    let rear = controls(ControllerKind::DualGrip)
        .into_iter()
        .find(|control| control.input == Input::Button(rear_button))
        .expect("DualGrip rear artwork")
        .geometry;
    let rear_y =
        picture.surface.top() + (rear[1] + rear[3] * 0.5) * picture.surface.width() / 730.0;
    ui.add_space((rear_y - 9.0 - ui.cursor().top()).max(16.0));
    dual_sidebar_button(app, ui, side, rear_button, picture, hover);
}

fn xbox_sidebar(
    app: &mut UI,
    ui: &mut egui::Ui,
    side: usize,
    picture: &Picture,
    hover: &mut ControlsHover,
) {
    let scale = picture.surface.width() / 730.0;
    let column = ui.available_rect_before_wrap();
    let source_y = |y: f32| picture.surface.top() + y * scale;
    let stick = XBOX_STICKS[side];
    let stick_y = source_y(stick[1] + stick[3] * 0.5) - if side == 1 { 10.0 } else { 0.0 };
    let horizontal_x = if side == 0 {
        column.right() - UI::axis_size().x * 0.5
    } else {
        column.left() + UI::axis_size().x * 0.5
    };
    let button_row = |app: &mut UI, ui: &mut egui::Ui, hover: &mut ControlsHover, index, y| {
        let stick_click_inset = if index == 7 || index == 8 {
            UI::axis_size().x * 0.5 - ui.spacing().icon_width * 0.5
        } else {
            0.0
        };
        let rect = Rect::from_min_size(
            egui::pos2(
                column.left() + if side == 1 { stick_click_inset } else { 0.0 },
                y - 9.0,
            ),
            Vec2::new(column.width() - stick_click_inset, 18.0),
        );
        let layout = if side == 0 {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };
        let mut child = ui.child_ui(rect, layout);
        sidebar_button(app, &mut child, index, picture, hover);
    };
    // Anchor bumper rows to their artwork and stick-click rows just above
    // the horizontal sliders. Checkbox icons stay at the inner column edge.
    button_row(
        app,
        ui,
        hover,
        if side == 0 { 9 } else { 10 },
        source_y(110.5),
    );
    button_row(app, ui, hover, 7 + side, stick_y - 46.0);
    app.stick_controls(ui, picture, side, egui::pos2(horizontal_x, stick_y), hover);
    if side == 1 {
        let rect = Rect::from_min_size(
            egui::pos2(column.left(), source_y(211.0) - 55.0),
            Vec2::new(126.0, 70.0),
        );
        let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
        face_controls(app, &mut child, picture, hover);
    } else {
        let rect = Rect::from_center_size(
            egui::pos2(
                column.right() - 44.0 - ui.spacing().icon_width + 31.0,
                source_y(295.0),
            ),
            Vec2::splat(62.0),
        );
        let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
        app.dpad_controls(&mut child, picture, hover);
    }
    let rear_y = source_y(379.0).max(stick_y + 62.0) - 30.0;
    for (offset, index) in if side == 0 { [11, 12] } else { [13, 14] }
        .iter()
        .enumerate()
    {
        button_row(app, ui, hover, *index, rear_y + offset as f32 * 24.0);
    }
    ui.advance_cursor_after_rect(Rect::from_min_max(
        column.min,
        egui::pos2(column.right(), rear_y + 34.0),
    ));
}

fn face_controls(app: &mut UI, ui: &mut egui::Ui, picture: &Picture, hover: &mut ControlsHover) {
    let (_, diamond) = ui.allocate_space(Vec2::new(126.0, 70.0));
    // Xbox face-button arrangement: Y above, X left, B right, A below.
    for (index, col, row) in [(3, 1, 0), (2, 0, 1), (1, 2, 1), (0, 1, 2)] {
        let rect = Rect::from_min_size(
            diamond.min + Vec2::new(col as f32 * 42.0, row as f32 * 24.0),
            Vec2::new(42.0, 22.0),
        );
        let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
        sidebar_button(app, &mut child, index, picture, hover);
    }
}

fn center_buttons(
    app: &mut UI,
    ui: &mut egui::Ui,
    picture: &Picture,
    hover: &mut ControlsHover,
    image: Rect,
) {
    let buttons = [4, 6, 5]; // View, Xbox, Menu.
    let width: f32 = buttons
        .iter()
        .map(|&index| {
            let galley = egui::WidgetText::from(app.buttons[index].name()).into_galley(
                ui,
                None,
                f32::INFINITY,
                egui::TextStyle::Button,
            );
            (ui.spacing().icon_width + ui.spacing().icon_spacing + galley.size().x)
                .max(ui.spacing().interact_size.x)
        })
        .sum::<f32>()
        + 2.0 * ui.spacing().item_spacing.x;
    let rect = Rect::from_center_size(
        egui::pos2(image.center().x, image.bottom() + 8.0 + 9.0),
        Vec2::new(width, 18.0),
    );
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal(|ui| {
            for index in buttons {
                sidebar_button(app, ui, index, picture, hover);
            }
        });
    });
}

pub fn show(app: &mut UI, ui: &mut egui::Ui) {
    let dual = app.kind == ControllerKind::DualGrip;
    let specs = controls(app.kind);
    let (available, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let side_width = 190.0;
    let image_available = Rect::from_min_max(
        available.min,
        available.max - Vec2::new(0.0, if dual { 0.0 } else { 32.0 }),
    );
    // Trim empty atlas margins and size the whole group from the image's
    // rendered dimensions, so widening the window does not separate its inputs.
    let crop = if dual {
        Rect::from_min_size(egui::pos2(105.0, 20.0), Vec2::new(520.0, 385.0))
    } else {
        Rect::from_min_size(egui::pos2(85.0, 8.0), Vec2::new(560.0, 402.0))
    };
    let scale = ((available.width() - 2.0 * (side_width + 8.0)) / crop.width())
        .min(image_available.height() / crop.height());
    let image = Rect::from_center_size(image_available.center(), crop.size() * scale);
    let row = Rect::from_center_size(
        image_available.center(),
        Vec2::new(
            image.width() + 2.0 * (side_width + 8.0),
            image_available.height(),
        ),
    );
    let surface = Rect::from_min_size(
        image.min - crop.min.to_vec2() * scale,
        Vec2::new(730.0, 435.0) * scale,
    );
    let map = |g: [f32; 4]| {
        Rect::from_min_size(
            surface.min + Vec2::new(g[0], g[1]) * scale,
            Vec2::new(g[2], g[3]) * scale,
        )
    };
    let painter = ui.painter().with_clip_rect(image);
    let mut picture = Picture {
        surface,
        painter,
        hovered_button: None,
        hovered_axes: [false; 5],
    };

    for (index, geometry) in if dual {
        &DUAL_SPECIAL[..]
    } else {
        &XBOX_STICKS[..]
    }
    .iter()
    .enumerate()
    {
        let rect = map(*geometry);
        let response = ui.interact(
            rect.expand(3.0 * scale),
            ui.id().with(("special", index)),
            Sense::click_and_drag(),
        );
        picture.hovered_axes[index] = controller_hovered(&response);
        if dual && index == 2 && controller_hovered(&response) {
            picture.hovered_button = Some(ButtonId::Joystick(13));
        }
        let stick_button = match (dual, index) {
            (true, 1) => Some(28),
            (false, 0 | 1) => Some(7 + index),
            _ => None,
        };
        if let Some(button) = stick_button {
            if controller_hovered(&response) {
                picture.hovered_button = Some(ButtonId::Joystick(button));
            }
            if response.clicked() {
                app.buttons[button].new_value = true;
                app.held_button = Some(button);
            }
        }
        if response.is_pointer_button_down_on() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let delta = (pointer - rect.center()) / (40.0 * scale);
                let x = (delta.x.clamp(-1.0, 1.0) * 100.0) as i8;
                let y = (delta.y.clamp(-1.0, 1.0) * 100.0) as i8;
                match index {
                    0 | 1 => {
                        app.image_axis(index * 2, x);
                        app.image_axis(index * 2 + 1, y);
                    }
                    2 => {
                        let direction = if x.abs().max(y.abs()) <= 50 {
                            None
                        } else if x.abs() > y.abs() {
                            Some(if x < 0 { 2 } else { 3 })
                        } else {
                            Some(if y < 0 { 0 } else { 1 })
                        };
                        press_direction(app, direction);
                    }
                    3 => app.image_axis(6, -x),
                    4 => app.image_axis(7, x),
                    _ => unreachable!(),
                }
            }
        }
    }
    if !dual {
        let rect = map([245.0, 261.0, 92.0, 71.0]);
        let response = ui.interact(rect, ui.id().with("xbox-dpad"), Sense::drag());
        picture.hovered_axes[2] = controller_hovered(&response);
        if response.is_pointer_button_down_on() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let delta = (pointer - rect.center()) / (rect.size() * 0.5);
                let largest = delta.x.abs().max(delta.y.abs());
                app.dpad_directions.fill(false);
                app.image_axis(
                    4,
                    if largest > 0.15 && delta.x.abs() >= largest * 0.45 {
                        delta.x.signum() as i8 * 100
                    } else {
                        0
                    },
                );
                app.image_axis(
                    5,
                    if largest > 0.15 && delta.y.abs() >= largest * 0.45 {
                        -delta.y.signum() as i8 * 100
                    } else {
                        0
                    },
                );
            }
        }
    }
    for (index, spec) in specs.iter().enumerate() {
        if matches!(spec.input, Input::Direction(_)) {
            continue;
        }
        let response = ui.interact(
            map(spec.geometry),
            ui.id().with(("input", index)),
            Sense::click_and_drag(),
        );
        if controller_hovered(&response) {
            match spec.input {
                Input::Button(index) => picture.hovered_button = Some(ButtonId::Joystick(index)),
                Input::Trigger(index) => picture.hovered_axes[index + 3] = true,
                Input::Direction(_) => picture.hovered_axes[2] = true,
            }
        }
        if response.is_pointer_button_down_on() {
            match spec.input {
                Input::Button(index) => {
                    app.buttons[index].new_value = true;
                    app.held_button = Some(index);
                }
                Input::Trigger(index) => app.image_axis(index + 6, 100),
                Input::Direction(index) => press_direction(app, Some(index)),
            }
        }
    }
    let mut hover = ControlsHover::default();
    for side in 0..2 {
        let rect = Rect::from_min_size(
            egui::pos2(
                if side == 0 {
                    row.left()
                } else {
                    row.right() - side_width
                },
                row.top(),
            ),
            Vec2::new(side_width, row.height()),
        );
        ui.allocate_ui_at_rect(rect, |ui| {
            let axis = side + 6;
            let size = UI::axis_size();
            let x = if side == 0 {
                rect.right() - size.x * 0.5
            } else {
                rect.left() + size.x * 0.5
            };
            let geometry = if dual {
                DUAL_SPECIAL[side + 3]
            } else {
                specs[side].geometry
            };
            let trigger_rect =
                Rect::from_center_size(egui::pos2(x, map(geometry).center().y), size);
            let mut child = ui.child_ui(trigger_rect, egui::Layout::top_down(egui::Align::Min));
            app.axis_control(&mut child, &picture, axis, &mut hover);
            if dual {
                egui::ScrollArea::vertical()
                    .id_source(("side", side))
                    .auto_shrink([false, true])
                    .show(ui, |ui| sidebar(app, ui, side, &picture, &mut hover));
            } else {
                sidebar(app, ui, side, &picture, &mut hover);
            }
        });
    }
    if !dual {
        center_buttons(app, ui, &picture, &mut hover, image);
    }
    if dual {
        for spec in &specs {
            if matches!(spec.input, Input::Button(21..=26)) {
                app.artwork.center_key(
                    &picture.painter,
                    map(spec.geometry),
                    active(app, spec.input),
                );
            } else {
                app.artwork.button(
                    &picture.painter,
                    map(spec.geometry),
                    spec.geometry,
                    active(app, spec.input),
                    spec.rear,
                    hovered(spec.input, &picture, &hover),
                );
            }
        }
        app.artwork.body(&picture.painter, surface);
        for spec in &specs {
            if spec.rear && (active(app, spec.input) || hovered(spec.input, &picture, &hover)) {
                app.artwork
                    .rear_lines(&picture.painter, map(spec.geometry), spec.geometry);
            }
        }
        for (index, geometry) in DUAL_SPECIAL.iter().enumerate() {
            let (x, y, angle, pressed) = match index {
                0 | 1 => (
                    app.axes[index * 2].new_value as f32 / 100.0,
                    app.axes[index * 2 + 1].new_value as f32 / 100.0,
                    0.0_f32,
                    index == 1 && app.buttons[28].new_value,
                ),
                2 => (
                    app.axes[4].new_value as f32 / 100.0,
                    -app.axes[5].new_value as f32 / 100.0,
                    0.0,
                    app.buttons[13].new_value,
                ),
                3 => (
                    0.0,
                    0.0,
                    -app.axes[6].new_value as f32 / 100.0 * 20.0,
                    false,
                ),
                _ => (0.0, 0.0, app.axes[7].new_value as f32 / 100.0 * 20.0, false),
            };
            let rect = map(*geometry)
                .translate(Vec2::new(x, y) * scale * if index == 2 { 8.0 } else { 14.0 });
            app.artwork.special(
                &picture.painter,
                rect,
                *geometry,
                angle.to_radians(),
                pressed || x.abs() > 0.02 || y.abs() > 0.02 || angle.abs() > 0.4,
                picture.hovered_axes[index]
                    || hover.axes[index]
                    || (index == 2 && hovered(Input::Button(13), &picture, &hover))
                    || (index == 1 && hovered(Input::Button(28), &picture, &hover)),
            );
        }
    } else {
        let art = app
            .xbox_artwork
            .as_ref()
            .expect("Xbox artwork loaded with device");
        for spec in &specs {
            let state = (
                active(app, spec.input),
                hovered(spec.input, &picture, &hover),
            );
            if let Input::Trigger(index) = spec.input {
                art.trigger(
                    &picture.painter,
                    map(spec.geometry),
                    spec.geometry,
                    app.axes[index + 6].new_value as f32 / 100.0,
                    index == 0,
                    state.1,
                );
                continue;
            }
            if !spec.rear || state.0 || state.1 {
                art.button(
                    &picture.painter,
                    map(spec.geometry),
                    spec.geometry,
                    spec.sheet,
                    state,
                );
            }
        }
        art.body(&picture.painter, surface);
        for spec in &specs {
            if spec.rear && (active(app, spec.input) || hovered(spec.input, &picture, &hover)) {
                art.paddle_lines(
                    &picture.painter,
                    map(spec.geometry),
                    spec.geometry,
                    spec.sheet == 1,
                );
            }
        }
        for (index, geometry) in XBOX_STICKS.iter().enumerate() {
            let delta = Vec2::new(
                app.axes[index * 2].new_value as f32,
                app.axes[index * 2 + 1].new_value as f32,
            ) / 100.0;
            let state = (
                delta.length_sq() > 0.0004 || app.buttons[7 + index].new_value,
                picture.hovered_axes[index]
                    || hover.axes[index]
                    || hovered(Input::Button(7 + index), &picture, &hover),
            );
            art.cap(
                &picture.painter,
                map(*geometry).translate(delta * scale * 16.0),
                *geometry,
                state,
            );
        }
    }
}
