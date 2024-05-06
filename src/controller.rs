use std::fmt::Debug;

use std::sync::mpsc;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, UinputAbsSetup,
};

const SLIDER_AXES: [(AbsoluteAxisType, &str); 6] = [
    (AbsoluteAxisType::ABS_X, "Left X"),
    (AbsoluteAxisType::ABS_Y, "Left Y"),
    (AbsoluteAxisType::ABS_RX, "Right X"),
    (AbsoluteAxisType::ABS_RY, "Right Y"),
    (AbsoluteAxisType::ABS_THROTTLE, "Throttle"),
    (AbsoluteAxisType::ABS_BRAKE, "Break"),
];

const BUTTONS: [(Key, &str); 13] = [
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

pub type AnalogAxis = Control<i8>;
pub type Button = Control<bool>;

pub type EventCode = u16;

pub fn build_uninput() -> anyhow::Result<(Box<[AnalogAxis]>, Box<[Button]>)> {
    let mut device = VirtualDeviceBuilder::new()?.name("Linux Virtual Joystick");

    let (event_sender, event_recv) = mpsc::channel();

    let abs_setup = AbsInfo::new(0, -100, 100, 0, 0, 1);
    let mut axes = Vec::with_capacity(SLIDER_AXES.len());
    for (axis, name) in SLIDER_AXES {
        let axis = UinputAbsSetup::new(axis, abs_setup);
        device = device.with_absolute_axis(&axis)?;
        axes.push(AnalogAxis::new(axis.code(), event_sender.clone(), name))
    }

    let mut buttons = Vec::with_capacity(SLIDER_AXES.len());
    let mut keys = AttributeSet::<Key>::new();
    for (button, name) in BUTTONS {
        keys.insert(button);
        buttons.push(Button::new(button.code(), event_sender.clone(), name))
    }
    let device = device.with_keys(&keys)?;

    let device = device.build()?;

    std::thread::spawn(|| device_thread(device, event_recv));

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
            self.new_value.controller_value(),
        );
        self.event_sender.send(control_value).unwrap();
        self.old_value = self.new_value.clone();
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub trait ControllerValue {
    fn controller_type() -> EventType;
    fn controller_value(&self) -> EventValue;
}

impl ControllerValue for i8 {
    fn controller_type() -> EventType {
        EventType::ABSOLUTE
    }
    fn controller_value(&self) -> EventValue {
        i32::from(*self)
    }
}

impl ControllerValue for bool {
    fn controller_type() -> EventType {
        EventType::KEY
    }

    fn controller_value(&self) -> EventValue {
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
        device.emit(&[new_event]).unwrap()
    }
}
