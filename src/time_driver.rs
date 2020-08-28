use amethyst::ecs::*;

pub enum TimeEvent {
    Resume,
    Stop,
    SetSpeed(f32),
}

#[derive(new)]
pub struct TimeDriverRes {
    reader: ReaderId<TimeEvent>,
    #[new(value = "1.0")]
    pub last_speed: f32,
}

system!(TimeDriver, |time: WriteExpect<'a, Time>,
        events: Read<'a, EventChannel<TimeEvent>>,
        reader: WriteExpect<'a, TimeDriverRes>| {
    for ev in events.read(&mut reader.reader) {
        match *ev {
            TimeEvent::Resume => if time.time_scale() != 0.0 {
                time.set_time_scale(reader.last_speed);
            },
            TimeEvent::Stop => {
                reader.last_speed = time.time_scale();
                time.set_time_scale(0);
            }
            TimeEvent::SetSpeed(s) => time.set_time_scale(s),
        }
    }
});

