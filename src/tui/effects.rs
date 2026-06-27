use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::time::Duration;
use tachyonfx::{fx, EffectManager, Motion};

pub struct Effects {
    manager: EffectManager<String>,
    startup_done: bool,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            manager: EffectManager::default(),
            startup_done: false,
        }
    }

    pub fn trigger_startup(&mut self, area: Rect) {
        if self.startup_done {
            return;
        }
        self.startup_done = true;
        let effect = fx::sweep_in(Motion::LeftToRight, 8, 0, Color::Black, 700u32)
            .with_area(area);
        self.manager.add_effect(effect);
    }

    pub fn trigger_file_open(&mut self, area: Rect) {
        let dissolve = fx::dissolve(250u32);
        let coalesce = fx::coalesce(350u32);
        let seq = fx::sequence(&[dissolve, coalesce]).with_area(area);
        self.manager.add_effect(seq);
    }

    pub fn trigger_save_flash(&mut self, area: Rect) {
        let effect = fx::fade_from(Color::Cyan, Color::Black, 500u32).with_area(area);
        self.manager.add_effect(effect);
    }

    pub fn trigger_focus_switch(&mut self, area: Rect) {
        let effect = fx::hsl_shift_fg([0.0, 0.0, 15.0], 200u32).with_area(area);
        self.manager.add_effect(effect);
    }

    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }

    pub fn process_frame(&mut self, elapsed: Duration, buf: &mut Buffer, area: Rect) {
        self.manager.process_effects(elapsed.into(), buf, area);
    }
}
