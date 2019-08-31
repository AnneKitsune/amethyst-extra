use amethyst::core::timing::Time;
use amethyst::core::*;
use amethyst::ecs::*;

use serde::Serialize;

use nphysics_ecs::ncollide::query::*;
use nphysics_ecs::events::*;
use nphysics_ecs::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, new)]
pub struct Grounded {
    #[new(value = "false")]
    pub ground: bool,
    #[new(default)]
    pub since: f64,
    pub distance_check: f32,
    /// Checks if the selected entity collides with the ground.
    #[serde(skip)]
    pub watch_entity: Option<Entity>,
}

impl Component for Grounded {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default, new)]
pub struct GroundCheckTag;

impl Component for GroundCheckTag {
    type Storage = DenseVecStorage<Self>;
}

/// T: ObjectType for collider checks
#[derive(new)]
pub struct GroundCheckerSystem<T> {
    pub collider_types: Vec<T>,
    //#[new(default)]
    //contact_reader: Option<ReaderId<EntityProximityEvent>>,
}

impl<'a, T: Component + PartialEq> System<'a> for GroundCheckerSystem<T> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Grounded>,
        ReadStorage<'a, T>,
        Read<'a, Time>,
        Read<'a, ProximityEvents>,
        ReadStorage<'a, PhysicsCollider<f32>>,
        ReadStorage<'a, GroundCheckTag>,
    );

    fn setup(&mut self, mut res: &mut Resources) {
        Self::SystemData::setup(&mut res);
        /*self.contact_reader = Some(
            res.fetch_mut::<EventChannel<ContactEvent<Entity, Point3<f32>>>>()
                .register_reader(),
        );*/
    }

    fn run(
        &mut self,
        (
            entities,
            transforms,
            mut grounded,
            _objecttypes,
            time,
            _contacts,
            colliders,
            ground_checks,
        ): Self::SystemData,
    ) {
        //let down = -Vector3::<f32>::y();
        for (_entity, _transform2, _player_collider, mut grounded) in
            (&*entities, &transforms, &colliders, &mut grounded).join()
        {
            //let mut ground = false;

            /*let ray = Ray3::new(Point3::from_vec(transform.translation), down);

            // For all in ray
            for (v, p) in query_ray(&*tree, ray) {
                // Not self and close enough
                if v.value != entity
                    && (transform.translation - Vector3::new(p.x, p.y, p.z)).magnitude()
                        <= grounded.distance_check
                {
                    // If we can jump off that type of collider
                    if let Some(obj_type) = objecttypes.get(v.value) {
                        if self.collider_types.contains(obj_type) {
                            ground = true;
                        }
                    }
                    //info!("hit bounding volume of {:?} at point {:?}", v.value, p);
                }
            }*/

            /*info!("run {:?}", entity);
            // Check for secondary collider if any
            for contact in contacts.read(&mut self.contact_reader.as_mut().unwrap()) {
                info!("Contact {:?} -> {:?}",contact.0, contact.1);
                // Here because we need to empty the contacts EventChannel
                if let Some(secondary) = grounded.watch_entity {
                    info!("Secondary");
                    if contact.0 == entity || contact.1 == entity {
                        // We hit our player... let's ignore that.
                        continue;
                    }
                    info!("tmp1");
                    if contact.0 != secondary && contact.1 != secondary {
                        // This has nothing to do with the secondary collider. Skip!
                        continue;
                    }
                    info!("type check");
                    let type1 = objecttypes.get(contact.0);
                    let type2 = objecttypes.get(contact.1);

                    if type1.is_none() || type2.is_none() {
                        continue;
                    }
                    info!("good to go");
                    // If we can jump off that type of collider
                    if self.collider_types.contains(type1.unwrap())
                        || self.collider_types.contains(type2.unwrap())
                    {
                        match contact.2.new_status {
                            // Collision with ground
                            Proximity::Intersecting | Proximity::WithinMargin => {
                                if ground && !grounded.ground {
                                    // Just grounded
                                    grounded.since = time.absolute_time_seconds();
                                }
                                grounded.ground = true;
                            },
                            // Collision stop
                            Proximity::Disjoint => {
                                grounded.ground = false;
                            }
                        }
                        ground = true;
                    }
                }


            }*/

            if let Some(secondary) = grounded.watch_entity {
                let transform = transforms
                    .get(secondary)
                    .expect("No transform component on secondary collider.");
                let feet_collider = colliders
                    .get(secondary)
                    .expect("No collider component on secondary collider.");
                info!(
                    "Gonna check for collision at player pos {:?}",
                    transform.translation()
                );

                let ground = (&*entities, &transforms, &colliders, &ground_checks)
                    .join()
                    .any(|(_entity, tr, collider, _)| {
                        if let Proximity::Intersecting = proximity(
                            &transform.isometry(),
                            &*feet_collider.shape.handle(),
                            &tr.isometry(),
                            &*collider.shape.handle(),
                            0.0,
                        ) {
                            warn!("COLLISION!!!");
                            true
                        } else {
                            false
                        }
                    });

                if ground && !grounded.ground {
                    // Just grounded
                    grounded.since = time.absolute_time_seconds();
                }
                grounded.ground = ground;
            }
        }
    }
}
