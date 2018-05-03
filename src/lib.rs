#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate amethyst;
#[macro_use]
extern crate serde;
extern crate ron;
#[macro_use]
extern crate log;
extern crate dirty;

use amethyst::assets::{Loader,AssetStorage,Handle};
use amethyst::renderer::{PosTex,VirtualKeyCode,Event,WindowEvent,KeyboardInput,Mesh};
use amethyst::ecs::prelude::{World,System,Read,Write,VecStorage,Entities,ReadStorage,WriteStorage,Component,Resources,Join,SystemData};
use amethyst::core::timing::Time;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::fs::File;
use std::io::Read as _IORead;
use std::io::Write as _IOWrite;
use dirty::Dirty;

/// Loads asset from the so-called asset packs
/// It caches assets which you can manually load or unload on demand, or load automatically.
/// It also tries to infer the type to load from the file extension.
///
/// Example:
/// If the folder structure looks like this
/// /assets/base/sprites/player.png
/// /assets/base/sounds/click.ogg
/// /assets/base/models/cube.obj
/// /assets/mod1/sprites/player.png
/// /assets/mod1/sounds/click.ogg
/// /assets/mod2/sounds/click.ogg
///
/// get("sprites/player.png") -> /assets/mod1/sprites/player.png
/// get("models/cube.obj") -> /assets/base/models/cube.obj
/// get("sounds/click.ogg") -> Unknown.
struct AssetLoader{
    base_path: String,
    default_pack: String,
}

impl Component for AssetLoader{
    type Storage = VecStorage<Self>;
}

pub fn gen_rectangle_mesh(
    w: f32,
    h: f32,
    loader: &Loader,
    storage: &AssetStorage<Mesh>,
) -> Handle<Mesh> {
    let verts = gen_rectangle_vertices(w, h);
    loader.load_from_data(verts.into(), (), &storage)
}
pub fn gen_rectangle_vertices(w: f32, h: f32) -> Vec<PosTex> {
    let data: Vec<PosTex> = vec![
        PosTex {
            position: [-w / 2., -h / 2., 0.],
            tex_coord: [0., 0.],
        },
        PosTex {
            position: [w / 2., -h / 2., 0.],
            tex_coord: [1., 0.],
        },
        PosTex {
            position: [w / 2., h / 2., 0.],
            tex_coord: [1., 1.],
        },
        PosTex {
            position: [w / 2., h / 2., 0.],
            tex_coord: [1., 1.],
        },
        PosTex {
            position: [-w / 2., h / 2., 0.],
            tex_coord: [0., 1.],
        },
        PosTex {
            position: [-w / 2., -h / 2., 0.],
            tex_coord: [0., 0.],
        },
    ];
    data
}

pub fn key_pressed_from_event(key: VirtualKeyCode, event: &Event) -> bool{
    match event {
        &Event::WindowEvent { ref event, .. } => match event {
            &WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(k),
                    ..
                },
                ..
            } => k == key,
            _ => false,
        },
        _ => false,
    }
}

pub fn window_closed(event: &Event) -> bool{
    match event {
        &Event::WindowEvent { ref event, .. } => match event {
            &WindowEvent::Closed => true,
            _ => false,
        },
        _ => false,
    }
}

/// If the tracked resource changes, this will be checked to make sure it is a proper time to save.
pub trait ShouldSave{
    fn save_ready(&self)->bool;
    fn set_save_ready(&mut self, ready: bool);
}

/// System used to automatically save a Resource T to a file.
/// On load, it will attempt to load it from the file and if it fails, it will use T::default().
pub struct AutoSaveSystem<T>{
    /// Absolute path.
    save_path: String,
    _phantom_data: PhantomData<T>
}

impl<T> AutoSaveSystem<T>{
    /// Save path is an absolute path.
    pub fn new(save_path: String)->Self{
        AutoSaveSystem{
            save_path,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a,T> System<'a> for AutoSaveSystem<T> where T: Serialize+DeserializeOwned+Default+ShouldSave+Send+Sync+'static{
    type SystemData = (Write<'a,Dirty<T>>,);
    fn setup(&mut self, res: &mut Resources){
        // attempt loading
        if let Ok(mut f) = File::open(&self.save_path){
            let mut c = String::new();
            if let Ok(_) = f.read_to_string(&mut c){
                if let Ok(o) = ron::de::from_str::<T>(&c){
                    res.insert(Dirty::new(o));
                }else{
                    error!("Failed to deserialize save file: {}.\nThe file might be corrupted.",self.save_path);
                }
            }else{
                error!("Failed to read content of save file: {}",self.save_path);
            }
        }else{
            warn!("Failed to load save file: {}. It will be created during the next save.",self.save_path);
        }
        Self::SystemData::setup(res);
    }
    fn run(&mut self, (mut d,): Self::SystemData){
        if let Some(v) = d.read_dirty(){
            let s = ron::ser::to_string(&v).expect(&format!("Unable to serialize the save struct for: {}",self.save_path));
            let mut f = File::create(&self.save_path);
            if f.is_ok(){
                let mut file = f.as_mut().ok().unwrap();
                let res = file.write_all(s.as_bytes());
                if res.is_err() {
                    error!("Failed to write serialized save data to the file. Error: {:?}",res.err().unwrap());
                }
            }else{
                error!("Failed to create or load the save file \"{}\". Error: {:?}",&self.save_path,&f.err().unwrap());
            }
        }

    }
}

pub struct DestroyAtTime{
    pub time: f64,
}

impl Component for DestroyAtTime{
    type Storage = VecStorage<Self>;
}

pub struct DestroyInTime{
    pub timer: f64,
}

impl Component for DestroyInTime{
    type Storage = VecStorage<Self>;
}

pub struct TimedDestroySystem;

impl<'a> System<'a> for TimedDestroySystem{
    type SystemData = (Entities<'a>,ReadStorage<'a,DestroyAtTime>,WriteStorage<'a,DestroyInTime>,Read<'a,Time>);
    fn run(&mut self, (entities,dat,mut dit,time): Self::SystemData){

        for (e,d) in (&*entities,&dat).join(){
            if time.absolute_time_seconds() > d.time {
                entities.delete(e);
            }
        }

        for (e,mut d) in (&*entities,&mut dit).join(){
            if d.timer <= 0.0 {
                entities.delete(e);
            }
            d.timer -= time.delta_seconds() as f64;
        }

    }
}

/*
  * = could do it in the engine directly
  BHop controller
  2D controllers
  load asset by name ("images/player.png"), infer which one to load using asset override system (modding)
  - {modname}/localisation/{player_lang}.txt
  *http calls utils
  item/inventory system
  *localisation

  DestroyAt component
  once the target time has past, destroy the entity

  AssetManager
*/
