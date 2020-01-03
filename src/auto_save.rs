use amethyst::ecs::*;

use dirty::Dirty;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::fs::File;

use std::io::Read as IORead;
use std::io::Write as IOWrite;

use std::marker::PhantomData;

/// If the tracked resource changes, this will be checked to make sure it is a proper time to save.
pub trait ShouldSave {
    fn save_ready(&self) -> bool;
    fn set_save_ready(&mut self, ready: bool);
}

/// System used to automatically save a Resource T to a file.
/// On load, it will attempt to load it from the file and if it fails, it will use T::default().
/// The resource in question will be wrapped into a `Dirty<T>` value inside of specs to keep track of changes made to the resource.
/// This `System` will save the resource each time there is a modification.
/// It is best used with resources that are modified less than once every second.
pub struct AutoSaveSystem<T> {
    /// Absolute path.
    save_path: String,
    _phantom_data: PhantomData<T>,
}

impl<T> AutoSaveSystem<T> 
where 
    T: Serialize + DeserializeOwned + Default + ShouldSave + Send + Sync + 'static,
{
    /// Create a new `AutoSaveSystem`.
    /// Save path is an absolute path.
    pub fn new(save_path: String) -> (Self, Option<Dirty<T>>) {
        // attempt loading
        let dirty = if let Ok(mut f) = File::open(&save_path) {
            let mut buf = String::new();
            if let Ok(_) = f.read_to_string(&mut buf) {
                if let Ok(o) = ron::de::from_str::<T>(&buf) {
                    Some(Dirty::new(o))
                } else {
                    error!(
                        "Failed to deserialize save file: {}.\nThe file might be corrupted.",
                        save_path
                    );
                    None
                }
            } else {
                error!("Failed to read content of save file: {}", save_path);
                None
            }
        } else {
            warn!(
                "Failed to load save file: {}. It will be created during the next save.",
                save_path
            );
            None
        };
        (AutoSaveSystem {
            save_path,
            _phantom_data: PhantomData,
        }, dirty)
    }
}

impl<'a, T> System<'a> for AutoSaveSystem<T>
where
    T: Serialize + DeserializeOwned + Default + ShouldSave + Send + Sync + 'static,
{
    type SystemData = (Write<'a, Dirty<T>>,);
    fn run(&mut self, (mut data,): Self::SystemData) {
        if data.dirty() {
            data.clear();
            let value = data.read();
            let string_data = ron::ser::to_string(&value).expect(&format!(
                "Unable to serialize the save struct for: {}",
                self.save_path
            ));
            let file = File::create(&self.save_path);
            match file {
                Ok(mut f) => {
                    // Write all serialized data to file.
                    let res = f.write_all(string_data.as_bytes());
                    if res.is_err() {
                        error!(
                            "Failed to write serialized save data to the file. Error: {:?}",
                            res.err().expect(
                                "unreachable: We know there is an error from the if clause."
                            )
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to create or load the save file \"{}\". Error: {:?}",
                        &self.save_path, e
                    );
                }
            }
        }
    }
}
