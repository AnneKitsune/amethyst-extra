use crate::movement::*;
use amethyst::core::math::{Vector2, Vector3};

use amethyst::core::timing::Time;
use amethyst::core::*;
use amethyst::ecs::*;
use amethyst::input::*;

use serde::Serialize;

use std::hash::Hash;

use std::marker::PhantomData;

use nphysics_ecs::*;

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct FpsMovement {
    /// The movement speed in units per second.
    pub speed: f32,
}

impl Component for FpsMovement {
    type Storage = DenseVecStorage<Self>;
}

/// The system that manages the fly movement.
/// Generic parameters are the parameters for the InputHandler.
#[derive(new)]
pub struct FpsMovementSystemSimple<A, B> {
    /// The name of the input axis to locally move in the x coordinates.
    /// Left and right.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    /// Forward and backward. Please note that -z is forward when defining your input configurations.
    forward_input_axis: Option<A>,
    _phantomdata: PhantomData<B>,
}

impl<'a, A, B> System<'a> for FpsMovementSystemSimple<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform>,
        Read<'a, InputHandler<A, B>>,
        ReadStorage<'a, FpsMovement>,
        WriteStorage<'a, DynamicBody>,
    );

    fn run(&mut self, (time, transforms, input, tags, mut rigid_bodies): Self::SystemData) {
        let x = get_input_axis_simple(&self.right_input_axis, &input);
        let z = get_input_axis_simple(&self.forward_input_axis, &input);

        let dir = Vector3::new(x, 0.0, z);
        if dir.magnitude() != 0.0 {
            for (transform, tag, rb) in (&transforms, &tags, &mut rigid_bodies).join() {
                let mut dir: Vector3<f32> = transform.rotation() * dir;
                dir = dir.normalize();
                rb.velocity.linear += dir * tag.speed * time.delta_seconds();
            }
        }
    }
}

/// The settings controlling how the entity controlled by the `BhopMovementSystem` will behave.
/// This is a component that you should add on the entity.
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct BhopMovement3D {
    /// False = Forces, True = Velocity
    pub absolute: bool,
    /// Use world coordinates XYZ.
    #[new(default)]
    pub absolute_axis: bool,
    /// Negates the velocity when pressing the key opposite to the current velocity.
    /// Effectively a way to instantly stop, even at high velocities.
    #[new(default)]
    pub counter_impulse: bool,
    /// Acceleration in unit/s² while on the ground.
    pub accelerate_ground: f32,
    /// Acceleration in unit/s² while in the air.
    pub accelerate_air: f32,
    /// The maximum ground velocity.
    pub max_velocity_ground: f32,
    /// The maximum air velocity.
    pub max_velocity_air: f32,
    /// Enables accelerating over maximumVelocity by airstrafing. Bunnyhop in a nutshell.
    pub allow_projection_acceleration: bool,
}

impl Component for BhopMovement3D {
    type Storage = DenseVecStorage<Self>;
}

/// The system that manages the first person movements (with added projection acceleration capabilities).
/// Generic parameters are the parameters for the InputHandler.
#[derive(new)]
pub struct BhopMovementSystem<A, B> {
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<A>,
    phantom_data: PhantomData<B>,
}

impl<'a, A, B> System<'a> for BhopMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, Time>,
        Read<'a, InputHandler<A, B>>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, BhopMovement3D>,
        ReadStorage<'a, Grounded>,
        WriteStorage<'a, DynamicBody>,
    );

    fn run(
        &mut self,
        (time, input, transforms, movements, groundeds, mut rigid_bodies): Self::SystemData,
    ) {
        let x = get_input_axis_simple(&self.right_input_axis, &input);
        let z = get_input_axis_simple(&self.forward_input_axis, &input);
        let input = Vector2::new(x, z);

        if input.magnitude() != 0.0 {
            for (transform, movement, grounded, mut rb) in
                (&transforms, &movements, &groundeds, &mut rigid_bodies).join()
            {
                let (acceleration, max_velocity) = if grounded.ground {
                    (movement.accelerate_ground, movement.max_velocity_ground)
                } else {
                    (movement.accelerate_air, movement.max_velocity_air)
                };

                // Global to local coords.
                let relative = transform.rotation().inverse() * rb.velocity.linear;

                let new_vel_rel = if movement.absolute {
                    // Absolute = We immediately set the maximum velocity without checking the max speed.
                    Vector3::new(input.x * acceleration, relative.y, input.y * acceleration)
                } else {
                    let mut wish_vel = relative;

                    if movement.counter_impulse {
                        wish_vel = counter_impulse(input, wish_vel);
                    }

                    wish_vel = accelerate_vector(
                        time.delta_seconds(),
                        input,
                        wish_vel,
                        acceleration,
                        max_velocity,
                    );
                    if !movement.allow_projection_acceleration {
                        wish_vel = limit_velocity(wish_vel, max_velocity);
                    }

                    wish_vel
                };

                // Global to local coords;
                let new_vel = transform.rotation() * new_vel_rel;

                // Assign the new velocity to the player
                rb.velocity.linear = new_vel;
            }
        }
    }
}
