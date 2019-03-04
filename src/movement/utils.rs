use amethyst::core::nalgebra::{Vector2, Vector3};

/// Accelerates the given `relative` vector by the given `acceleration` and `input`.
/// The `maximum_velocity` is only taken into account for the projection of the acceleration vector on the `relative` vector.
/// This allows going over the speed limit by performing what is called a "strafe".
/// If your velocity is forward and have an input accelerating you to the right, the projection of
/// the acceleration vector over your current velocity will be 0. This means that the acceleration vector will be applied fully,
/// even if this makes the resulting vector's magnitude go over `max_velocity`.
pub fn accelerate_vector(
    delta_time: f32,
    input: Vector2<f32>,
    rel: Vector3<f32>,
    acceleration: f32,
    max_velocity: f32,
) -> Vector3<f32> {
    let mut o = rel;
    let input3 = Vector3::new(input.x, 0.0, input.y);
    let rel_flat = Vector3::new(rel.x, 0.0, rel.z);
    if input3.magnitude() > 0.0 {
        let proj = rel_flat.dot(&input3.normalize());
        let mut accel_velocity = acceleration * delta_time as f32;
        if proj + accel_velocity > max_velocity {
            accel_velocity = max_velocity - proj;
        }
        if accel_velocity > 0.0 {
            let add_speed = input3 * accel_velocity;
            o += add_speed;
        }
    }
    o
}

/// Completely negates the velocity of a specific axis if an input is performed in the opposite direction.
pub fn counter_impulse(input: Vector2<f32>, relative_velocity: Vector3<f32>) -> Vector3<f32> {
    let mut o = relative_velocity;
    if (input.x < 0.0 && relative_velocity.x > 0.001)
        || (input.x > 0.0 && relative_velocity.x < -0.001)
    {
        o = Vector3::new(0.0, relative_velocity.y, relative_velocity.z);
    }
    if (input.y < 0.0 && relative_velocity.z < -0.001)
        || (input.y > 0.0 && relative_velocity.z > 0.001)
    {
        o = Vector3::new(relative_velocity.x, relative_velocity.y, 0.0);
    }
    o
}

/// Limits the total velocity so that its magnitude doesn't exceed `maximum_velocity`.
/// If you are using the `accelerate_vector` function, calling this will ensure that air strafing
/// doesn't allow you to go over the maximum velocity, while still keeping fluid controls.
pub fn limit_velocity(vec: Vector3<f32>, maximum_velocity: f32) -> Vector3<f32> {
    let v_flat = Vector2::new(vec.x, vec.z).magnitude();
    if v_flat > maximum_velocity && maximum_velocity != 0.0 {
        let ratio = maximum_velocity / v_flat;
        return Vector3::new(vec.x * ratio, vec.y, vec.z * ratio);
    }
    vec
}
