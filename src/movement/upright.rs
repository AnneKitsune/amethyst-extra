


use amethyst::core::nalgebra::UnitQuaternion;








use amethyst::core::*;
use amethyst::ecs::*;






























//use crossterm::screen::RawScreen;






#[derive(Default, new)]
pub struct UprightTag;

impl Component for UprightTag {
    type Storage = DenseVecStorage<Self>;
}

/// BROKEN, DO NOT USE
#[derive(Default, new)]
pub struct ForceUprightSystem;

impl<'a> System<'a> for ForceUprightSystem {
    type SystemData = (WriteStorage<'a, Transform>, ReadStorage<'a, UprightTag>);

    fn run(&mut self, (mut transforms, uprights): Self::SystemData) {
        (&mut transforms, &uprights)
            .join()
            .map(|t| t.0)
            .for_each(|tr| {
                // roll, pitch, yaw
                let angles = tr.rotation().euler_angles();
                info!("Force upright angles: {:?}", angles);
                let new_quat = UnitQuaternion::from_euler_angles(0.0, angles.1, 0.0);
                *tr.rotation_mut() = new_quat;
            });
    }
}