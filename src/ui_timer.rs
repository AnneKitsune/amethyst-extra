











use amethyst::core::timing::Time;

use amethyst::ecs::*;


use amethyst::ui::{UiText};





























//use crossterm::screen::RawScreen;






pub struct UiTimer {
    pub start: f64,
}

impl Component for UiTimer {
    type Storage = VecStorage<Self>;
}

pub struct UiTimerSystem;

impl<'a> System<'a> for UiTimerSystem {
    type SystemData = (
        ReadStorage<'a, UiTimer>,
        WriteStorage<'a, UiText>,
        Read<'a, Time>,
    );
    fn run(&mut self, (timers, mut texts, time): Self::SystemData) {
        for (timer, mut text) in (&timers, &mut texts).join() {
            let t = time.absolute_time_seconds() - timer.start;
            text.text = t.to_string(); // Simply show seconds for now.
        }
    }
}