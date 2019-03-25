use amethyst::ecs::*;

use amethyst::ui::UiText;

use std::marker::PhantomData;

pub trait UiAutoText: Component {
    fn get_text(&self) -> String;
}

#[derive(Default)]
pub struct UiAutoTextSystem<T> {
    phantom: PhantomData<T>,
}

impl<'a, T> System<'a> for UiAutoTextSystem<T>
where
    T: Component + UiAutoText,
{
    type SystemData = (ReadStorage<'a, T>, WriteStorage<'a, UiText>);
    fn run(&mut self, (autotexts, mut texts): Self::SystemData) {
        for (autotext, mut text) in (&autotexts, &mut texts).join() {
            text.text = autotext.get_text();
        }
    }
}
