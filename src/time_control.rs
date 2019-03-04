





use amethyst::shrev::EventChannel;





use amethyst::core::timing::Time;

use amethyst::ecs::*;
use amethyst::input::*;











use serde::Serialize;



use std::hash::Hash;














//use crossterm::screen::RawScreen;






#[derive(new, Debug, Serialize, Deserialize)]
pub struct ManualTimeControl<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    pub play_action_key: T,
    pub stop_action_key: T,
    pub half_action_key: T,
    pub double_action_key: T,
}

#[derive(new, Debug, Default)]
pub struct ManualTimeControlSystem<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    #[new(default)]
    event_reader: Option<ReaderId<InputEvent<T>>>,
}

impl<'a, T> System<'a> for ManualTimeControlSystem<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Write<'a, Time>,
        Read<'a, EventChannel<InputEvent<T>>>,
        ReadExpect<'a, ManualTimeControl<T>>,
    );

    fn run(&mut self, (mut time, events, time_control): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                InputEvent::ActionPressed(key) => {
                    if *key == time_control.play_action_key {
                        time.set_time_scale(1.0);
                    } else if *key == time_control.stop_action_key {
                        time.set_time_scale(0.0);
                    } else if *key == time_control.half_action_key {
                        let time_scale = time.time_scale();
                        time.set_time_scale(time_scale * 0.5);
                    } else if *key == time_control.double_action_key {
                        let time_scale = time.time_scale();
                        time.set_time_scale(time_scale * 2.0);
                    }
                }
                _ => {}
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(
            res.fetch_mut::<EventChannel<InputEvent<T>>>()
                .register_reader(),
        );
    }
}