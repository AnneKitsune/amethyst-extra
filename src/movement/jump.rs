


use crate::movement::ground::Grounded;
use amethyst::core::nalgebra::Vector3;







use amethyst::core::timing::Time;

use amethyst::ecs::*;
use amethyst::input::*;









use partial_function::*;




















//use crossterm::screen::RawScreen;




use nphysics_ecs::*;

#[derive(Default, new)]
pub struct Jump {
    pub absolute: bool,
    pub check_ground: bool,
    pub jump_force: f32,
    pub auto_jump: bool,
    #[new(value = "0.3333")]
    pub jump_cooldown: f64,
    #[new(value = "0.1")]
    pub input_cooldown: f64,
    /// Multiplier. Time can go in the negatives.
    #[new(default)]
    pub jump_timing_boost: Option<PartialFunction<f64, f32>>,
    #[new(default)]
    pub last_jump: f64,
    #[new(default)]
    pub last_jump_offset: f64,
}

impl Component for Jump {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct JumpSystem {
    /// The last time the system considered a valid jump input.
    last_logical_press: f64,
    /// Was the jump key pressed last frame?
    input_hold: bool,
    /// The last time we physically pressed the jump key.
    last_physical_press: f64,
}

impl<'a> System<'a> for JumpSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Grounded>,
        WriteStorage<'a, Jump>,
        Read<'a, Time>,
        Read<'a, InputHandler<String, String>>,
        WriteStorage<'a, DynamicBody>,
    );

    fn run(
        &mut self,
        (entities, mut grounded, mut jumps, time, input, mut rigid_bodies): Self::SystemData,
    ) {
        if let Some(true) = input.action_is_down("jump") {
            if !self.input_hold {
                // We just started pressing the key. Registering time.
                self.last_physical_press = time.absolute_time_seconds();
                self.input_hold = true;
            }

            for (entity, mut jump, mut rb) in (&*entities, &mut jumps, &mut rigid_bodies).join() {
                // Holding the jump key on a non-auto jump controller.
                if self.input_hold && !jump.auto_jump {
                    continue;
                }

                // The last time we jumped wasn't long enough ago
                if time.absolute_time_seconds() - self.last_logical_press < jump.input_cooldown
                {
                    continue;
                }
                self.last_logical_press = time.absolute_time_seconds();

                // If we need to check for it, verify that we are on the ground.
                let mut grounded_since = time.absolute_time_seconds();
                if jump.check_ground {
                    if let Some(ground) = grounded.get(entity) {
                        if !ground.ground {
                            continue;
                        }
                        grounded_since = ground.since;
                    } else {
                        continue;
                    }
                }

                if time.absolute_time_seconds() - jump.last_jump > jump.jump_cooldown {
                    // Jump!
                    jump.last_jump = time.absolute_time_seconds();
                    // Offset for jump. Positive = time when we jumped AFTER we hit the ground.
                    jump.last_jump_offset = grounded_since - self.last_physical_press;

                    let multiplier = if let Some(ref curve) = jump.jump_timing_boost {
                        curve.eval(jump.last_jump_offset).unwrap_or(1.0)
                    } else {
                        1.0
                    };

                    if !jump.absolute {
                        rb.velocity.linear +=
                            Vector3::<f32>::y() * jump.jump_force * multiplier;
                    } else {
                        let (x, z) = {
                            let v = rb.velocity.linear;
                            (v.x, v.z)
                        };
                        rb.velocity.linear = Vector3::new(x, jump.jump_force * multiplier, z);
                    }
                }
                if let Some(ref mut ground) = grounded.get_mut(entity) {
                    ground.ground = false;
                }
            }
        } else {
            // The jump key was released.
            self.input_hold = false;
        }
    }
}