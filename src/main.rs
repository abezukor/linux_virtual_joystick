use eframe::egui;

mod controller;

fn main() -> anyhow::Result<()> {
    let (axes, buttons) = controller::build_uninput()?;

    eframe::run_native(
        "Linux Virtual Joystick",
        Default::default(),
        Box::new(|_cc| Box::new(UI::new(axes, buttons))),
    )
    .unwrap();
    Ok(())
}

struct UI {
    axes: Box<[controller::AnalogAxis]>,
    button: Box<[controller::Button]>,
}

impl UI {
    pub fn new(axes: Box<[controller::AnalogAxis]>, button: Box<[controller::Button]>) -> Self {
        Self { axes, button }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for axis in self.axes.iter_mut() {
                let name = axis.name();
                ui.add(egui::Slider::new(&mut axis.new_value, -100..=100).text(name));
            }

            for button in self.button.iter_mut() {
                let name = button.name();
                ui.add(egui::Checkbox::new(&mut button.new_value, name));
            }
        });

        for axis in self.axes.iter_mut() {
            axis.new_value();
        }

        for button in self.button.iter_mut() {
            button.new_value()
        }
    }
}
