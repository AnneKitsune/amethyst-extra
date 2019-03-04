
#[derive(new, Debug, Serialize, Deserialize)]
pub struct NoClip<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    pub toggle_action_key: T,
    #[new(default)]
    #[serde(skip)]
    pub(crate) noclip_entity: Option<Entity>,
    #[new(default)]
    #[serde(skip)]
    pub(crate) previous_active_camera: Option<Entity>,
    #[new(default)]
    pub(crate) active: bool,
}

#[derive(Default, new, Serialize, Deserialize, Clone, Copy)]
pub struct NoClipTag;

impl Component for NoClipTag {
    type Storage = NullStorage<Self>;
}

/// Toggle the noclip camera.
/// Spawns the noclip fly entity at the position of the current main camera.
/// Forces the entity's camera to always be the primary one.
/// When untoggling, deletes the entity and sets the main camera to whatever last one was set as primary.
/// (including the one changed during the noclipping)
#[derive(new, Debug, Default)]
pub struct NoClipToggleSystem<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    #[new(default)]
    event_reader: Option<ReaderId<InputEvent<T>>>,
}

impl<'a, T> System<'a> for NoClipToggleSystem<T>
where
    T: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Entities<'a>,
        Read<'a, EventChannel<InputEvent<T>>>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, GlobalTransform>,
        WriteStorage<'a, FlyControlTag>,
        WriteStorage<'a, Camera>,
        Write<'a, ActiveCamera>,
        WriteExpect<'a, NoClip<T>>,
        WriteStorage<'a, NoClipTag>,
    );

    fn run(
        &mut self,
        (
            entities,
            events,
            mut transforms,
            mut _global_transforms,
            mut fly_control_tags,
            mut cameras,
            mut active_camera,
            mut noclip_res,
            mut noclips,
        ): Self::SystemData,
    ) {
        if active_camera.entity != noclip_res.noclip_entity {
            noclip_res.previous_active_camera = active_camera.entity;
        }

        // TODO: AutoFov support

        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                InputEvent::ActionPressed(key) => {
                    if *key == noclip_res.toggle_action_key {
                        if !noclip_res.active {
                            // Enable noclip
                            let entity = entities.create();
                            let transform = Transform::default(); // TODO: get global position of current main entity.
                            transforms.insert(entity, transform).unwrap();
                            fly_control_tags.insert(entity, FlyControlTag).unwrap();
                            cameras
                                .insert(entity, Camera::standard_3d(800.0, 600.0))
                                .unwrap(); // TODO: clone main camera if available.
                            noclips.insert(entity, NoClipTag).unwrap();

                            active_camera.entity = Some(entity);
                            noclip_res.noclip_entity = Some(entity);

                            noclip_res.active = true;
                        } else {
                            // Disable noclip

                            // get noclip entity
                            if let Some(entity) = noclip_res.noclip_entity {
                                if let Err(err) = entities.delete(entity) {
                                    error!("Noclip is enabled, but there is no noclip entity in the world! {}", err);
                                }

                                active_camera.entity = noclip_res.previous_active_camera.clone();
                                noclip_res.noclip_entity = None;
                            } else {
                                error!("Noclip is enabled, but there is no noclip entity in the world!");
                            }

                            noclip_res.active = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(
            res.fetch_mut::<EventChannel<InputEvent<T>>>()
                .register_reader(),
        );
    }
}