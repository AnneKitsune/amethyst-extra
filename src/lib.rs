extern crate amethyst;
#[macro_use]
extern crate serde;
extern crate ron;
#[macro_use]
extern crate log;
extern crate dirty;

use amethyst::assets::{Loader,AssetStorage,Handle,Format,Asset,SimpleFormat};
use amethyst::renderer::{PosTex,VirtualKeyCode,Event,WindowEvent,KeyboardInput,Mesh,ObjFormat};
use amethyst::ecs::prelude::{World,System,Read,Write,VecStorage,Entities,ReadStorage,WriteStorage,Component,Resources,Join,SystemData,Dispatcher};
use amethyst::ecs::storage::NullStorage;
use amethyst::core::timing::Time;
use amethyst::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::fs::File;
use std::fs;
use std::path::Path;
use std::io::Read as _IORead;
use std::io::Write as _IOWrite;
use dirty::Dirty;
use std::collections::HashMap;


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
/// get::<Texture>("sprites/player.png") -> /assets/mod1/sprites/player.png
/// get::<Mesh>("models/cube.obj") -> /assets/base/models/cube.obj
/// get::<Audio>("sounds/click.ogg") -> Unknown.
///
///
/// yet to resolve: asset pack ordering & deps
pub struct AssetLoader{
    /// Should end with a /
    base_path: String,
    default_pack: String,
    asset_packs: Vec<String>,
}

impl AssetLoader{

    pub fn new(base_path: String, default_pack: String) -> Self{
        let mut al = AssetLoader{
            base_path,
            default_pack,
            asset_packs: Vec::new(),
        };
        al.get_asset_packs();
        al
    }

    pub fn resolve_path(&self, path: &str) -> Option<String> {
        let mut res: Option<String> = None;

        // Try to get from default path
        res = self.resolve_path_for_pack(path,&self.default_pack);

        // Try to find overrides
        for p in &self.asset_packs{
            if p != &self.default_pack{
                if let Some(r) = self.resolve_path_for_pack(path,&p){
                    res = Some(r);
                }
            }
        }

        res
    }
    fn resolve_path_for_pack(&self, path: &str, pack: &str) -> Option<String> {
        let abs = self.base_path.to_owned() + pack + "/" + &path.to_owned();
        let path = Path::new(&abs);
        if path.exists(){
            Some(abs.clone())
        }else{
            None
        }
    }
    pub fn get_asset_packs(&mut self) -> &Vec<String>{
        let mut buf: Option<Vec<String>> = None;
        if self.asset_packs.len() == 0{
            if let Ok(elems) = fs::read_dir(&self.base_path){
                buf = Some(elems.map(|e|{
                    e.unwrap().path().to_str().unwrap()[self.base_path.len()..].to_string()
                }).collect());
            }else{
                error!("Failed to find base_path directory for asset loading: {}",self.base_path);
            }
        }

        if let Some(v) = buf{
            self.asset_packs = v;
        }

        &self.asset_packs
    }
    pub fn get_asset_handle<T>(path: &str, ali: &AssetLoaderInternal<T>) -> Option<Handle<T>>{
        ali.assets.get(path).cloned()
    }
    pub fn get_asset<'a,T>(path: &str, ali: &AssetLoaderInternal<T>, storage: &'a AssetStorage<T>) -> Option<&'a T> where T: Asset{
        if let Some(h) = AssetLoader::get_asset_handle::<T>(path,ali){
            storage.get(&h)
        }else{
            None
        }
    }

    pub fn get_asset_or_load<'a,T,F>(&mut self, path: &str,format: F, options: F::Options, ali: &mut AssetLoaderInternal<T>, storage: &'a mut AssetStorage<T>,loader: Loader) -> Option<&'a T>
        where T: Asset, F: SimpleFormat<T>+Clone+Send+Sync+'static{
        if let Some(h) = AssetLoader::get_asset_handle::<T>(path,ali){
            return storage.get(&h);
            //return Some(a);
        }
        if let Some(h) = self.load::<T,F>(path, format, options,ali,storage,loader){
            return storage.get(&h);
        }
        None
    }

    pub fn load<T,F>(&self, path: &str, format: F, options: F::Options, ali: &mut AssetLoaderInternal<T>, storage: &mut AssetStorage<T>, loader: Loader) -> Option<Handle<T>>
        where T: Asset, F: SimpleFormat<T>+Clone+Send+Sync+'static{
        if let Some(p) = self.resolve_path(path){
            let handle: Handle<T> = loader.load(p,format,options,(),storage);
            ali.assets.insert(String::from(path),handle.clone());
            return Some(handle);
        }
        None
    }
    pub fn unload<T>(path: &str, ali: &mut AssetLoaderInternal<T>){
        ali.assets.remove(path);
    }

    /*pub fn load_from_extension<T>(&mut self,path: &str,ali: &mut AssetLoaderInternal<T>, storage: &mut AssetStorage<T>, loader: Loader) -> Option<Handle<T>> where T: Asset{
        let ext = AssetLoader::extension_from_path(path);
        match ext{
            "obj" => Some(self.load::<Mesh,ObjFormat>(path,ObjFormat,ali,storage,loader)),
            _ => None,
        }
    }

    pub fn auto_load_from_extension(&mut self,path: &str,res: Resources){
        let ext = AssetLoader::extension_from_path(path);
        match ext{
            "obj" => Some(self.load_from_extension::<Mesh>(ext,res.fetch_mut::<AssetLoaderInternal<Mesh>>(),res.fetch_mut::<AssetStorage<Mesh>>(),res.fetch())),
            _ => None,
        };
    }*/

    /*pub fn extension_from_path(path: &str) -> &str{
        path.split(".").as_slice().last().clone()
    }*/

}

impl Component for AssetLoader{
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct AssetLoaderInternal<T>{
    /// Map path to asset handle.
    pub assets: HashMap<String,Handle<T>>,
}

impl<T> Component for AssetLoaderInternal<T> where T: Send+Sync+'static{
    type Storage = VecStorage<Self>;
}

/*pub trait AssetToFormat<T> where T: Sized{
    fn get_format() -> Format<T>;
}

impl AssetToFormat<Mesh> for Mesh{
    fn get_format() -> Format<Mesh>{
        ObjFormat
    }
}*/




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
            &WindowEvent::CloseRequested => true,
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

/*pub struct EmptyState;
impl<'a,'b> State<GameData<'a,'b>> for EmptyState{

}

#[derive(Default)]
pub struct RemoveOnStateChange;
impl Component for RemoveOnStateChange{
    type Storage = NullStorage<Self>;
}

pub struct ComplexState<'a,'b,T> where T: State<GameData<'a,'b>>{
    internal: T,
    dispatch: Option<Dispatcher<'static,'static>>,
}

impl<'a,'b,T> ComplexState<'a,'b,T> where T: State<GameData<'a,'b>>{

    pub fn new(state: T, dispatch: Option<Dispatcher<'static,'static>>) -> Self {
        ComplexState{
            internal: state,
            dispatch,
        }
    }
}

impl<'a,'b,T> State for ComplexState<T> where T: State<GameData<'a,'b>>{
    //forward everything to internal state, but add operations to remove collected entities
    fn on_start(&mut self, mut world: &mut World) {
        if let Some(dis) = self.dispatch.as_mut(){
            dis.setup(&mut world.res);
        }
        self.internal.on_start(&mut world);
    }

    fn update(&mut self, mut world: &mut World) -> Trans {
        if let Some(dis) = self.dispatch.as_mut(){
            dis.dispatch(&mut world.res);
        }
        self.internal.update(&mut world)
    }

    fn handle_event(&mut self, mut world: &mut World, event: Event) -> Trans {
        self.internal.handle_event(&mut world, event)
    }
}


pub struct NavigationButton{
    pub target: fn() -> Trans,
}

impl Component for NavigationButton{
    type Storage = VecStorage<Self>;
}*/

/*
  * = could do it in the engine directly
  BHop controller
  2D controllers
  load asset by name ("images/player.png"), infer which one to load using asset override system (modding)
  - {modname}/localisation/{player_lang}.txt
  *http calls utils
  item/inventory system

*/
#[cfg(test)]
mod tests {
    #[test]
    fn asset_loader_resolve_path() {
        let mut al = AssetLoader::new(format!("{}/assets/", env!("CARGO_MANIFEST_DIR")),"base");
        assert_eq!(al.resolve_path("test/test.txt"),"assets/base/test/test.txt")
    }
    #[test]
    fn asset_loader_resolve_path_override() {
        let mut al = AssetLoader::new(format!("{}/assets/", env!("CARGO_MANIFEST_DIR")),"base");
        assert_eq!(al.resolve_path("test/test2.txt"),"assets/override/test/test2.txt")
    }
}