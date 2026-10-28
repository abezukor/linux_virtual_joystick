use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, BusType, EventType, InputEvent, InputId, Key,
    UinputAbsSetup,
};
use std::fmt::Debug;
use std::sync::mpsc;

const SLIDER_AXES: [(AbsoluteAxisType, &str); 8] = [
    (AbsoluteAxisType::ABS_X, "Joy Lt/Rt"),
    (AbsoluteAxisType::ABS_Y, "Joy Up/Dn"),
    (AbsoluteAxisType::ABS_Z, "Joy Lt/Rt"),
    (AbsoluteAxisType::ABS_RZ, "Joy Up/Dn"),
    (AbsoluteAxisType::ABS_HAT0X, "D-pad X"),
    (AbsoluteAxisType::ABS_HAT0Y, "D-pad Y"),
    (AbsoluteAxisType::ABS_RX, "Left Shoulder"),
    (AbsoluteAxisType::ABS_RY, "Right Shoulder"),
];

// Match the raw button order in the GCS's existing Active5 SDL mapping.
const BUTTONS: [(Key, &str); 14] = [
    (Key::BTN_EAST, "A"),
    (Key::BTN_SOUTH, "B"),
    (Key::BTN_C, "C / Misc"),
    (Key::BTN_TL2, "Rear"),
    (Key::BTN_TR2, "Rear"),
    (Key::BTN_Z, "Z / Misc"),
    (Key::BTN_NORTH, "LB"),
    (Key::BTN_WEST, "RB"),
    (Key::BTN_TL, "LT"),
    (Key::BTN_TR, "RT"),
    (Key::BTN_SELECT, "Select"),
    (Key::BTN_START, "Start"),
    (Key::BTN_MODE, "Mode"),
    (Key::BTN_THUMBL, "D-pad Press"),
];

pub type AnalogAxis = Control<i8>;
pub type Button = Control<bool>;

pub type EventCode = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerKind {
    Active5,
    DualGrip,
    Xbox,
    LinuxVirtualJoystick,
}

impl ControllerKind {
    pub const ALL: [Self; 4] = [
        Self::Active5,
        Self::DualGrip,
        Self::Xbox,
        Self::LinuxVirtualJoystick,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Active5 => "SRoC Tab Active5",
            Self::DualGrip => "SRoC DualGrip",
            Self::Xbox => "Xbox Elite 2",
            Self::LinuxVirtualJoystick => "Linux Virtual Joystick",
        }
    }

    pub fn dpad_center(self) -> Option<usize> {
        if matches!(self, Self::Xbox | Self::LinuxVirtualJoystick) {
            None
        } else {
            Some(13)
        }
    }
}

pub fn build_uninput(kind: ControllerKind) -> anyhow::Result<(Box<[AnalogAxis]>, Box<[Button]>)> {
    // Match the identities and raw layouts in the GCS's existing SDL database.
    let (name, vendor, product, version) = match kind {
        ControllerKind::Active5 => ("UXV Technologies SRoC Active5", 0x372c, 0x0105, 0x0111),
        ControllerKind::DualGrip => ("UXV Technologies SRoC DualGrip", 0x372c, 0x0111, 0x0111),
        ControllerKind::Xbox => ("Microsoft Xbox One Elite 2", 0x045e, 0x0b00, 0x050d),
        ControllerKind::LinuxVirtualJoystick => return build_original_uninput(),
    };
    let mut device = VirtualDeviceBuilder::new()?
        .name(name)
        .input_id(InputId::new(BusType::BUS_USB, vendor, product, version));
    let (event_sender, event_recv) = mpsc::channel();

    let mut axis_definitions = SLIDER_AXES;
    if kind == ControllerKind::Xbox {
        // xpad order: X/Y, Z (LT), RX/RY (right stick), RZ (RT).
        axis_definitions[2].0 = AbsoluteAxisType::ABS_RX;
        axis_definitions[3].0 = AbsoluteAxisType::ABS_RY;
        axis_definitions[6] = (AbsoluteAxisType::ABS_Z, "LT");
        axis_definitions[7] = (AbsoluteAxisType::ABS_RZ, "RT");
    }
    let mut axes = Vec::with_capacity(axis_definitions.len());
    for (index, (code, name)) in axis_definitions.iter().copied().enumerate() {
        let limit = if is_hat(code.0) { 1 } else { 100 };
        let minimum = if kind == ControllerKind::Xbox && index >= 6 {
            0
        } else {
            -limit
        };
        device = device.with_absolute_axis(&UinputAbsSetup::new(
            code,
            AbsInfo::new(0, minimum, limit, 0, 0, 0),
        ))?;
        axes.push(AnalogAxis::new(code.0, event_sender.clone(), name));
    }

    let mut definitions = BUTTONS.to_vec();
    if kind == ControllerKind::DualGrip {
        // Keep the UI's familiar SRoC positions while using DualGrip's raw keys.
        definitions[0].0 = Key::BTN_SOUTH;
        definitions[1].0 = Key::BTN_EAST;
        definitions[3].0 = Key::BTN_SELECT;
        definitions[4].0 = Key::BTN_START;
        definitions[6].0 = Key::BTN_TL;
        definitions[7].0 = Key::BTN_TR;
        definitions[8].0 = Key::BTN_TL2;
        definitions[9].0 = Key::BTN_TR2;
        definitions[10] = (Key::BTN_WEST, "Misc");
        definitions[11] = (Key::BTN_NORTH, "Misc");
        // Extended HID buttons are forwarded by the GCS's DualGrip backend.
        // Reserve raw slots 14-20, then expose F1-F4 and the two center keys.
        for index in 14..29 {
            let name = match index {
                21 => "F1",
                22 => "F2",
                23 => "F3",
                24 => "F4",
                25 => "Center Left",
                26 => "Center Right",
                28 => "RS",
                _ => "Reserved",
            };
            definitions.push((Key::new(Key::BTN_TRIGGER_HAPPY1.0 + index - 14), name));
        }
    } else if kind == ControllerKind::Xbox {
        definitions = vec![
            (Key::BTN_SOUTH, "A"),
            (Key::BTN_EAST, "B"),
            (Key::BTN_NORTH, "X"),
            (Key::BTN_WEST, "Y"),
            (Key::BTN_SELECT, "View"),
            (Key::BTN_START, "Menu"),
            (Key::BTN_MODE, "Xbox"),
            (Key::BTN_THUMBL, "LS"),
            (Key::BTN_THUMBR, "RS"),
            (Key::BTN_TL, "LB"),
            (Key::BTN_TR, "RB"),
            (Key::BTN_TRIGGER_HAPPY5, "P3"),
            (Key::BTN_TRIGGER_HAPPY7, "P4"),
            (Key::BTN_TRIGGER_HAPPY6, "P1"),
            (Key::BTN_TRIGGER_HAPPY8, "P2"),
        ];
    }
    let mut buttons = Vec::with_capacity(definitions.len());
    let mut keys = AttributeSet::<Key>::new();
    for (button, name) in definitions {
        keys.insert(button);
        buttons.push(Button::new(button.0, event_sender.clone(), name));
    }
    let device = device.with_keys(&keys)?.build()?;
    std::thread::spawn(|| device_thread(device, event_recv));
    Ok((axes.into_boxed_slice(), buttons.into_boxed_slice()))
}

// Preserve the original device and control order from commit 4ce3d72f.
fn build_original_uninput() -> anyhow::Result<(Box<[AnalogAxis]>, Box<[Button]>)> {
    const AXES: [(AbsoluteAxisType, &str); 6] = [
        (AbsoluteAxisType::ABS_X, "Left X"),
        (AbsoluteAxisType::ABS_Y, "Left Y"),
        (AbsoluteAxisType::ABS_RX, "Right X"),
        (AbsoluteAxisType::ABS_RY, "Right Y"),
        (AbsoluteAxisType::ABS_THROTTLE, "Throttle"),
        (AbsoluteAxisType::ABS_BRAKE, "Break"),
    ];
    const KEYS: [(Key, &str); 13] = [
        (Key::BTN_SOUTH, " A (South)"),
        (Key::BTN_EAST, "B (East)"),
        (Key::BTN_WEST, "Y (West)"),
        (Key::BTN_NORTH, "X North"),
        (Key::BTN_TL, "LT"),
        (Key::BTN_TR, "RT"),
        (Key::BTN_TL2, "LB"),
        (Key::BTN_TR2, "RB"),
        (Key::BTN_SELECT, "Select / View"),
        (Key::BTN_START, "Menu / Start"),
        (Key::BTN_THUMBL, "Left Thumbstick"),
        (Key::BTN_THUMBR, "Right Thumbstick"),
        (Key::BTN_MODE, "Mode"),
    ];
    let mut device = VirtualDeviceBuilder::new()?.name("Linux Virtual Joystick");
    let (sender, receiver) = mpsc::channel();
    let setup = AbsInfo::new(0, -100, 100, 0, 0, 1);
    let mut axes = Vec::with_capacity(AXES.len());
    for (code, name) in AXES {
        device = device.with_absolute_axis(&UinputAbsSetup::new(code, setup))?;
        axes.push(AnalogAxis::new(code.0, sender.clone(), name));
    }
    let mut buttons = Vec::with_capacity(KEYS.len());
    let mut keys = AttributeSet::<Key>::new();
    for (code, name) in KEYS {
        keys.insert(code);
        buttons.push(Button::new(code.0, sender.clone(), name));
    }
    let device = device.with_keys(&keys)?.build()?;
    std::thread::spawn(|| device_thread(device, receiver));
    Ok((axes.into_boxed_slice(), buttons.into_boxed_slice()))
}

type EventValue = i32;

#[derive(Debug)]
pub struct Control<T: Default + PartialEq + Clone + ControllerValue + Debug> {
    event_code: EventCode,
    old_value: T,
    pub new_value: T,
    event_sender: mpsc::Sender<InputEvent>,
    name: &'static str,
}

impl<T: Default + PartialEq + ControllerValue + Clone + Debug> Control<T> {
    pub fn new(
        event_code: EventCode,
        event_sender: mpsc::Sender<InputEvent>,
        name: &'static str,
    ) -> Self {
        Self {
            event_code,
            old_value: T::default(),
            new_value: T::default(),
            event_sender,
            name,
        }
    }

    pub fn new_value(&mut self) {
        if self.new_value == self.old_value {
            return;
        }
        let control_value = InputEvent::new(
            T::controller_type(),
            self.event_code,
            self.new_value.controller_value(self.event_code),
        );
        let _ = self.event_sender.send(control_value);
        self.old_value = self.new_value.clone();
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub trait ControllerValue {
    fn controller_type() -> EventType;
    fn controller_value(&self, event_code: EventCode) -> EventValue;
}

fn is_hat(event_code: EventCode) -> bool {
    event_code == AbsoluteAxisType::ABS_HAT0X.0 || event_code == AbsoluteAxisType::ABS_HAT0Y.0
}

impl ControllerValue for i8 {
    fn controller_type() -> EventType {
        EventType::ABSOLUTE
    }
    fn controller_value(&self, event_code: EventCode) -> EventValue {
        if is_hat(event_code) {
            // The UI slider is analog; the real SRoC D-pad is a digital hat.
            let direction = if *self < -50 {
                -1
            } else if *self > 50 {
                1
            } else {
                0
            };
            // Vertical UI sliders increase upward; Linux hats use -1 for up.
            if event_code == AbsoluteAxisType::ABS_HAT0Y.0 {
                -direction
            } else {
                direction
            }
        } else {
            i32::from(*self)
        }
    }
}

impl ControllerValue for bool {
    fn controller_type() -> EventType {
        EventType::KEY
    }

    fn controller_value(&self, _event_code: EventCode) -> EventValue {
        i32::from(*self)
    }
}

// This could also be done using async rust, but that seemed to be overkill for this project.
fn device_thread(mut device: VirtualDevice, events: mpsc::Receiver<InputEvent>) {
    loop {
        let Ok(new_event) = events.recv() else {
            //Sender dropped, so app is being closed.
            break;
        };
        if device.emit(&[new_event]).is_err() {
            break;
        }
    }
}
