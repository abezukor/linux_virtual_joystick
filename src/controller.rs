use uinput::event::Event;
use uinput::event::Event::{Controller, Absolute};
use uinput::event::Absolute::{Position, Wheel};
use uinput::event::controller::Controller::GamePad;
use uinput::event::controller::GamePad as GamePad_Events;

pub struct ControllerInterface{
    button_events: [Event; 15],
    axes_events: [Event; 6],
    device: uinput::Device
}
impl ControllerInterface{
    pub fn new() -> ControllerInterface {
        //All the button Events
        let button_events = [
            Controller(GamePad(GamePad_Events::A)),
            Controller(GamePad(GamePad_Events::B)),
            Controller(GamePad(GamePad_Events::C)),
            Controller(GamePad(GamePad_Events::X)),
            Controller(GamePad(GamePad_Events::Y)),
            Controller(GamePad(GamePad_Events::Z)),
            Controller(GamePad(GamePad_Events::TL)),
            Controller(GamePad(GamePad_Events::TR)),
            Controller(GamePad(GamePad_Events::TL2)),
            Controller(GamePad(GamePad_Events::TR2)),
            Controller(GamePad(GamePad_Events::Select)),
            Controller(GamePad(GamePad_Events::Start)),
            Controller(GamePad(GamePad_Events::Mode)),
            Controller(GamePad(GamePad_Events::ThumbL)),
            Controller(GamePad(GamePad_Events::ThumbR)),
        ];
        //All the slider events
        let axes_events = [
            Absolute(Position(uinput::event::absolute::Position::X)), 
            Absolute(Position(uinput::event::absolute::Position::Y)),
            Absolute(Position(uinput::event::absolute::Position::RX)),
            Absolute(Position(uinput::event::absolute::Position::RY)),
            Absolute(Wheel(uinput::event::absolute::Wheel::Throttle)),
            Absolute(Wheel(uinput::event::absolute::Wheel::Brake))
        ];
        //Make a new device, and error out cleanly.
        let mut dev_builder = match uinput::default(){
            Ok(build) => build.name("Linux Virtual Joystick").unwrap(),
            Err(_e) => {panic!("
Uinput file not found, you may need to enable the uinput kernel module with:
    modprobe uinput
If you are still getting this error, make sure that your user has rw access to /dev/uinput.\n")}
        };
        //Add the sliders
        for i in 0..6{
            dev_builder = dev_builder.event(axes_events[i]).unwrap();
        }
        //Add the buttons
        for i in 0..15{
            dev_builder = dev_builder.event(button_events[i]).unwrap();
        }
        //Make a new interface
        ControllerInterface {
            button_events: button_events,
            axes_events: axes_events,
            device: dev_builder.create().unwrap()
        }
    }

    pub fn button_change(&mut self, num_button: usize, new_state: bool){
        //Changes the state of a controller button
        self.device.send(self.button_events[num_button], new_state as i32).unwrap();
        self.device.synchronize().unwrap();
    }
    pub fn axes_change(&mut self, num_axes: usize, new_value: f64){
        //Changes the state of a controller axes
        //Axes go to i16 limits, so scale from float
        let axis_value: i32 = (new_value*(i16::MAX as f64)) as i32;
        self.device.send(self.axes_events[num_axes], axis_value).unwrap();
        self.device.synchronize().unwrap()
    }
}
