use amethyst::controls::HideCursor;
use amethyst::controls::WindowFocus;
use amethyst::core::math::UnitQuaternion;
use amethyst::winit::{DeviceEvent, Event};
use amethyst::shrev::EventChannel;

use amethyst::core::*;
use amethyst::ecs::*;

use serde::Serialize;

use std::hash::Hash;

use std::marker::PhantomData;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RotationControl {
    pub mouse_accum_x: f32,
    pub mouse_accum_y: f32,
}

impl Component for RotationControl {
    type Storage = DenseVecStorage<Self>;
}

/// The system that manages the view rotation.
/// Controlled by the mouse.
/// Put the RotationControl component on the Camera. The Camera should be a child of the player collider entity.
#[derive(Debug)]
pub struct FPSRotationRhusicsSystem<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
    event_reader: ReaderId<Event>,
}

impl<A, B> FPSRotationRhusicsSystem<A, B> 
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    pub fn new(sensitivity_x: f32, sensitivity_y: f32, world: &mut World) -> Self {
        <Self as System>::SystemData::setup(world);
        let event_reader = world.fetch_mut::<EventChannel<Event>>().register_reader();
        FPSRotationRhusicsSystem {sensitivity_x, sensitivity_y, _marker1: PhantomData, _marker2: PhantomData, event_reader}
    }
}

impl<'a, A, B> System<'a> for FPSRotationRhusicsSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Entities<'a>,
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, RotationControl>,
        Read<'a, WindowFocus>,
        Read<'a, HideCursor>,
        ReadStorage<'a, Parent>,
    );

    fn run(
        &mut self,
        (
            entities,
            events,
            mut transforms,
            mut rotation_controls,
            focus,
            hide,
            parents,
        ): Self::SystemData,
    ) {
        let focused = focus.is_focused;
        let win_events = events
            .read(&mut self.event_reader)
            .collect::<Vec<_>>();
        if !win_events.iter().any(|e| match e {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { .. },
                ..
            } => true,
            _ => false,
        }) {
            // Patch until we have physics constraints
            for (rotation_control, parent) in (&rotation_controls, &parents).join() {
                // Player collider
                if let Some(tr) = transforms.get_mut(parent.entity) {
                    *tr.rotation_mut() =
                        UnitQuaternion::from_euler_angles(0.0, rotation_control.mouse_accum_x, 0.0);
                }
            }
        } else {
            for event in win_events {
                if focused && hide.hide {
                    if let Event::DeviceEvent { ref event, .. } = *event {
                        if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                            // camera
                            for (entity, mut rotation_control, parent) in
                                (&*entities, &mut rotation_controls, &parents).join()
                            {
                                rotation_control.mouse_accum_x -= x as f32 * self.sensitivity_x;
                                rotation_control.mouse_accum_y -= y as f32 * self.sensitivity_y;
                                // Limit maximum vertical angle to prevent locking the quaternion and/or going upside down.
                                // rotation_control.mouse_accum_y = rotation_control.mouse_accum_y.max(-89.5).min(89.5);
                                rotation_control.mouse_accum_y = rotation_control
                                    .mouse_accum_y
                                    .max(-std::f64::consts::FRAC_PI_2 as f32 + 0.001)
                                    .min(std::f64::consts::FRAC_PI_2 as f32 - 0.001);
                                // Camera
                                if let Some(tr) = transforms.get_mut(entity) {
                                    *tr.rotation_mut() = UnitQuaternion::from_euler_angles(
                                        rotation_control.mouse_accum_y,
                                        0.0,
                                        0.0,
                                    );
                                }
                                // Player collider
                                if let Some(tr) = transforms.get_mut(parent.entity) {
                                    *tr.rotation_mut() = UnitQuaternion::from_euler_angles(
                                        0.0,
                                        rotation_control.mouse_accum_x,
                                        0.0,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
