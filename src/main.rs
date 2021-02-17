extern crate uinput;
extern crate gtk;

/*use std::{thread};
use std::time::Duration;
*/

use std::sync::{Arc, Mutex};

use gtk::{prelude::*};

mod controller;
use controller::ControllerInterface;

fn main() {
    //Make a sharable ControllerInterface
    let con_interface = Arc::new(Mutex::new(ControllerInterface::new()));

    //initialize GTK with glade file
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("lvj_ui.glade");
    let builder = gtk::Builder::from_string(glade_src);
    //Get the GTK window
    let window: gtk::Window = builder.get_object("LVJ-Window").unwrap();
    
    //Initialize the handlers for all the sliders
    for i in 0..6{
        let axis_adjustment: gtk::Adjustment = builder.get_object(format!("Analog{}",i+1).as_str()).unwrap();
        let current_controller_interface = con_interface.clone();
        axis_adjustment.connect_value_changed( move |adj| {
            //println!("Axis {} changed to {}.", i, adj.get_value());
            current_controller_interface.lock().unwrap().axes_change(i, adj.get_value());
        });
    }

    //Initialize the handlers for all the buttons
    for i in 0..14{
        let current_button: gtk::ToggleButton = builder.get_object(format!("BTN{}",i+1).as_str()).unwrap();
        let current_controller_interface = con_interface.clone();
        current_button.connect_clicked(move |btn| {
            //println!("Button {} is {}.",i, btn.get_active());
            current_controller_interface.lock().unwrap().button_change(i, btn.get_active())
        });
    }
    //Show thee window
    window.show_all();

    //Let the window be closed
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    
    gtk::main();
    

}
