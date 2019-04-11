use crate::movement::ground::Grounded;
use amethyst::core::math::Vector3;

use amethyst::core::timing::Time;

use amethyst::ecs::*;

use serde::Serialize;

use nphysics_ecs::*;

/// The way friction is applied.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum FrictionMode {
    /// The velocity is reduced by a fixed amount each second (deceleration).
    Linear,
    /// The velocity is reduced by a fraction of the current velocity.
    /// A value of 0.2 means that approximatively 20% of the speed will be lost each second.
    /// Since it is not calculated as an integration but as discrete values, the actual slowdown will vary slightly from case to case.
    Percent,
}

/// Component you add to your entities to apply a ground friction.
/// What the friction field does is dependent on the choosen `FrictionMode`.
#[derive(Serialize, Deserialize, Clone, Debug, new)]
pub struct GroundFriction3D {
    /// The amount of friction speed loss by second.
    pub friction: f32,
    /// The way friction is applied.
    pub friction_mode: FrictionMode,
    /// The time to wait after touching the ground before applying the friction.
    pub ground_time_before_apply: f64,
}

impl Component for GroundFriction3D {
    type Storage = DenseVecStorage<Self>;
}

/// Applies friction (slows the velocity down) according to the `GroundFriction3D` component of your entity.
/// Your entity also needs to have a `Grounded` component (and the `GroundCheckerSystem` added to your dispatcher) to detect the ground.
/// It also needs to have a NextFrame<Velocity3<f32>> component. This is added automatically by rhusics when creating a dynamic physical entity.
pub struct GroundFrictionSystem;

impl<'a> System<'a> for GroundFrictionSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Grounded>,
        ReadStorage<'a, GroundFriction3D>,
        WriteStorage<'a, DynamicBody>,
    );

    fn run(&mut self, (time, groundeds, frictions, mut rigid_bodies): Self::SystemData) {
        fn apply_friction_single(v: f32, friction: f32) -> f32 {
            if v.abs() <= friction {
                return 0.0;
            }
            v - friction
        }
        for (grounded, friction, mut rb) in (&groundeds, &frictions, &mut rigid_bodies).join() {
            if grounded.ground
                && time.absolute_time_seconds() - grounded.since
                    >= friction.ground_time_before_apply
            {
                let (x, y, z) = {
                    let v = rb.velocity.linear;
                    (v.x, v.y, v.z)
                };
                match friction.friction_mode {
                    FrictionMode::Linear => {
                        let slowdown = friction.friction * time.delta_seconds();
                        rb.velocity.linear = Vector3::new(
                            apply_friction_single(x, slowdown),
                            y,
                            apply_friction_single(z, slowdown),
                        );
                    }
                    FrictionMode::Percent => {
                        let coef = friction.friction * time.delta_seconds();
                        rb.velocity.linear = Vector3::new(
                            apply_friction_single(x, x * coef),
                            y,
                            apply_friction_single(z, z * coef),
                        );
                    }
                }
            }
        }
    }
}
