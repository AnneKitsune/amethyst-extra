use ::amethyst::shrev::EventChannel;

use ::amethyst::core::timing::Time;

use ::amethyst::ecs::*;
use ::amethyst::input::*;

use serde::Serialize;

#[derive(new, Debug, Serialize, Deserialize)]
pub struct ManualTimeControl<T>
where
    T: BindingTypes,
{
    pub play_action_key: T::Action,
    pub stop_action_key: T::Action,
    pub half_action_key: T::Action,
    pub double_action_key: T::Action,
}

#[derive(Debug)]
pub struct ManualTimeControlSystem<T>
where
    T: BindingTypes,
{
    event_reader: ReaderId<InputEvent<T>>,
}

impl<T> ManualTimeControlSystem<T>
where
    T: BindingTypes,
{
    pub fn new(world: &mut World) -> Self {
        <Self as System>::SystemData::setup(world);
        let event_reader = world
            .fetch_mut::<EventChannel<InputEvent<T>>>()
            .register_reader();
        ManualTimeControlSystem { event_reader }
    }
}

impl<'a, T> System<'a> for ManualTimeControlSystem<T>
where
    T: BindingTypes,
{
    type SystemData = (
        Write<'a, Time>,
        Read<'a, EventChannel<InputEvent<T>>>,
        ReadExpect<'a, ManualTimeControl<T>>,
    );

    fn run(&mut self, (mut time, events, time_control): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
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
}
