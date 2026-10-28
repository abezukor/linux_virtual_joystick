use eframe::egui::{self, Context, Response, Vec2};

#[derive(Default)]
pub struct ResizeHover {
    previous_size: Option<Vec2>,
    blocked: bool,
    resume_after: f64,
}

impl ResizeHover {
    pub fn begin_frame(&mut self, ctx: &Context) {
        let enabled = ctx.input(|input| {
            self.update(
                input.screen_rect().size(),
                input.time,
                input.pointer.delta() != Vec2::ZERO,
                input.pointer.any_pressed(),
            )
        });
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("controller-hover-enabled"), enabled));
    }

    fn update(&mut self, size: Vec2, time: f64, moved: bool, pressed: bool) -> bool {
        let resized = self.previous_size.is_some_and(|previous| previous != size);
        self.previous_size = Some(size);
        if resized {
            self.blocked = true;
            // Ignore pointer events generated while the window manager is
            // still resizing. Waiting alone never activates a new hover.
            self.resume_after = time + 0.12;
        } else if pressed || (moved && time >= self.resume_after) {
            self.blocked = false;
        }
        !self.blocked
    }
}

pub fn enabled(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(egui::Id::new("controller-hover-enabled")))
        .unwrap_or(true)
}

pub fn controller_hovered(response: &Response) -> bool {
    enabled(&response.ctx) && response.hovered()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINAL: Vec2 = Vec2::new(1200.0, 620.0);
    const RESIZED: Vec2 = Vec2::new(1600.0, 620.0);

    #[test]
    fn stationary_pointer_does_not_hover_controls_moved_by_resize() {
        let mut hover = ResizeHover::default();
        assert!(hover.update(ORIGINAL, 0.0, false, false));
        assert!(!hover.update(RESIZED, 0.1, false, false));
        assert!(!hover.update(RESIZED, 2.0, false, false));
        assert!(hover.update(RESIZED, 2.1, true, false));
    }

    #[test]
    fn resize_motion_does_not_reenable_hover() {
        let mut hover = ResizeHover::default();
        hover.update(ORIGINAL, 0.0, false, false);
        assert!(!hover.update(RESIZED, 0.1, true, false));
        assert!(!hover.update(RESIZED, 0.15, true, false));
        assert!(!hover.update(ORIGINAL, 0.2, true, false));
        assert!(!hover.update(ORIGINAL, 0.3, true, false));
        assert!(hover.update(ORIGINAL, 0.4, true, false));
    }

    #[test]
    fn clicking_after_resize_restores_feedback_immediately() {
        let mut hover = ResizeHover::default();
        hover.update(ORIGINAL, 0.0, false, false);
        hover.update(RESIZED, 0.1, false, false);
        assert!(hover.update(RESIZED, 0.11, false, true));
    }

    #[test]
    fn hover_stays_enabled_without_a_resize() {
        let mut hover = ResizeHover::default();
        assert!(hover.update(ORIGINAL, 0.0, false, false));
        assert!(hover.update(ORIGINAL, 0.1, true, false));
        assert!(hover.update(ORIGINAL, 0.2, false, false));
    }
}
