extern crate amethyst;
#[macro_use]
extern crate serde;
extern crate ron;
#[macro_use]
extern crate log;
extern crate dirty;
extern crate termion;
extern crate fern;

use amethyst::assets::{Loader,AssetStorage,Handle,Format,Asset,SimpleFormat};
use amethyst::animation::AnimationBundle;
use amethyst::audio::{AudioBundle,SourceHandle};
use amethyst::ui::{UiBundle,UiText};
use amethyst::input::*;
use amethyst::core::*;
use amethyst::renderer::*;
use amethyst::ecs::*;
use amethyst::ecs::storage::NullStorage;
use amethyst::core::cgmath::Ortho;
use amethyst::core::timing::Time;
use amethyst::prelude::*;
use amethyst::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::fs::File;
use std::fs;
use std::path::Path;
use std::io::Read as IORead;
use std::io::Write as IOWrite;
use std::io;
use dirty::Dirty;
use std::iter::Cycle;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::time::Duration;
use std::thread::sleep;
use std::hash::Hash;
use amethyst::core::cgmath::{Vector4,SquareMatrix};


use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::async_stdin;
use std::io::{stdin, stdout};

/// Loads asset from the so-called asset packs
/// It caches assets which you can manually load or unload on demand.
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
/// resolve_path("sprites/player.png") -> /assets/mod1/sprites/player.png
/// resolve_path("models/cube.obj") -> /assets/base/models/cube.obj
/// resolve_path("sounds/click.ogg") -> Unknown.
pub struct AssetLoader{
    base_path: String,
    default_pack: String,
    asset_packs: Vec<String>,
}

impl AssetLoader{

    pub fn new(base_path: &str, default_pack: &str) -> Self{
        let mut al = AssetLoader{
            base_path: AssetLoader::sanitize_path_trail_only(&base_path),
            default_pack: AssetLoader::sanitize_path(&default_pack),
            asset_packs: Vec::new(),
        };
        al.get_asset_packs();
        al
    }

    fn sanitize_path_trail_only(path: &str) -> String {
        let mut out = path.to_string();
        let mut chars = path.chars();
        let last = chars.last().unwrap();
        if last == '/' {
          let idx = out.len() - 1;
          out.remove(idx);
        }
        out
    }
    
    fn sanitize_path(path: &str) -> String {
        let mut out = path.to_string();
        let mut chars = path.chars();
        let first = chars.next().expect("An empty path was specified!");
        let last = chars.last().unwrap();
        if first == '/' {
          out.remove(0);
        }
        if last == '/' {
          let idx = out.len() - 1;
          out.remove(idx);
        }
        out
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
        let abs = self.base_path.to_owned()+ "/" + pack + "/" + &path.to_owned();
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
                    let path = &e.unwrap().path();
                    let tmp = &path.to_str().unwrap()[self.base_path.len()..];
                    AssetLoader::sanitize_path(&tmp)
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
    
    pub fn get_asset_or_load<'a,T,F>(&mut self, path: &str,format: F, options: F::Options, ali: &mut AssetLoaderInternal<T>, storage: &'a mut AssetStorage<T>,loader: &Loader) -> Option<&'a T>
        where T: Asset, F: Format<T>+'static{
        if let Some(h) = AssetLoader::get_asset_handle::<T>(path,ali){
            return storage.get(&h);
            //return Some(a);
        }
        if let Some(h) = self.load::<T,F>(path, format, options,ali,storage,loader){
            return storage.get(&h);
        }
        None
    }
    
    pub fn load<T,F>(&self, path: &str, format: F, options: F::Options, ali: &mut AssetLoaderInternal<T>, storage: &mut AssetStorage<T>, loader: &Loader) -> Option<Handle<T>>
        where T: Asset, F: Format<T>+'static{
        if let Some(handle) = AssetLoader::get_asset_handle(path,ali){
            return Some(handle);
        }
        if let Some(p) = self.resolve_path(path){
            let handle: Handle<T> = loader.load(p,format,options,(),storage);
            ali.assets.insert(String::from(path),handle.clone());
            return Some(handle);
        }
        None
    }
    
    /// Only removes the internal Handle<T>. To truly unload the asset, you need to drop all handles that you have to it.
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

#[cfg(test)]
mod test{
  use ::*;
  
  fn load_asset_loader() -> AssetLoader{
    AssetLoader::new(&format!("{}/test/assets",env!("CARGO_MANIFEST_DIR")), "main")
  }
  
  #[test]
  fn path_sanitisation(){
    AssetLoader::new(&format!("{}/test/assets/", env!("CARGO_MANIFEST_DIR")), "/base/");
  }
  
    #[test]
    fn asset_loader_resolve_unique_main() {
        let mut asset_loader = load_asset_loader();
        assert_eq!(asset_loader.resolve_path("config/unique"),Some(format!("{}/test/assets/main/config/unique",env!("CARGO_MANIFEST_DIR")).to_string()))
    }
    
    #[test]
    fn asset_loader_resolve_unique_other() {
        let mut asset_loader = load_asset_loader();
        assert_eq!(asset_loader.resolve_path("config/uniqueother"),Some(format!("{}/test/assets/mod1/config/uniqueother",env!("CARGO_MANIFEST_DIR")).to_string()))
    }
    
    #[test]
    fn asset_loader_resolve_path_override_single() {
        let mut asset_loader = load_asset_loader();
        assert_eq!(asset_loader.resolve_path("config/ov1"),Some(format!("{}/test/assets/mod1/config/ov1",env!("CARGO_MANIFEST_DIR")).to_string()))
    }
    
    #[test]
    fn asset_loader_resolve_path_override_all() {
        let mut asset_loader = load_asset_loader();
        assert_eq!(asset_loader.resolve_path("config/ovall"),Some(format!("{}/test/assets/mod2/config/ovall",env!("CARGO_MANIFEST_DIR")).to_string()))
    }
    
    #[test]
    fn termion_test() {
        start_logger();
        let mut stdin = async_stdin().bytes();
        let mut stdout = stdout().into_raw_mode().unwrap();
        let mut buf = Vec::<u8>::new();
        print!("{}",termion::cursor::Hide);
        loop {
            //print!("random_garbage_");
            let (term_width,term_height) = termion::terminal_size().ok().expect("Failed to get terminal size.");
            //println!("random stuff \r");
            warn!("random stuff");
            while let Some(b) = stdin.next(){
                info!("\r{:?} <- Char entered \r", b);
                //println!("this should hopefully switch ({:?}) \r",b);
                if let Ok(b) = b {
                    if b == 3{
                        print!("{}{}",termion::cursor::Show,termion::clear::CurrentLine);
                        //print!("\r\nreset\r\n");
                        stdout.flush().unwrap();
                        drop(stdout);
                        std::process::exit(0); // Ctrl+C = exit immediate
                    }else if b == 13 {
                        println!("BUFFER: {:?}",buf);
                        buf.clear();
                    }else{
                        buf.push(b);
                    }
                }
            }
            
            // Print entry line at the bottom
            print_first_line(&buf);
            clear_second_line();
            stdout.flush().unwrap();
            
            sleep(Duration::from_millis(100));
        }
    }
    fn reset_input_line(){
        
    }
    fn clear_second_line_str()->String{
        let (_,term_height) = termion::terminal_size().ok().expect("Failed to get terminal size.");
        format!("{}{}",termion::cursor::Goto(1,term_height-1),termion::clear::CurrentLine)
    }
    fn clear_second_line(){
        print!("{}",clear_second_line_str());
    }
    fn print_first_line(buf: &Vec<u8>){
        let (_,term_height) = termion::terminal_size().ok().expect("Failed to get terminal size.");
        print!("{}{}>{}",termion::cursor::Goto(1,term_height),termion::clear::CurrentLine,String::from_utf8_lossy(&buf));
    }
    
    // On print: goto line 1, clear, write, \r\n, clear write line 1
    
    pub fn start_logger() {
        let color_config = fern::colors::ColoredLevelConfig::new();

        fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color}[{level}][{target}] {message}{color_reset}\r\n{clear}",
                color = format!(
                        "\x1B[{}m",
                        color_config.get_color(&record.level()).to_fg_str()
                    ),
                level = record.level(),
                target = record.target(),
                message = message,
                color_reset = "\x1B[0m",
                clear = clear_second_line_str(),
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()
        .unwrap_or_else(|_| {
            error!("Global logger already set, amethyst-extra logger not used!")
        });
    }
}

/*pub trait AssetToFormat<T> where T: Sized{
    fn get_format() -> Format<T>;
}

impl AssetToFormat<Mesh> for Mesh{
    fn get_format() -> Format<Mesh>{
        ObjFormat
    }
}*/



/// Generates a rectangle 2d mesh.
pub fn gen_rectangle_mesh(
    w: f32,
    h: f32,
    loader: &Loader,
    storage: &AssetStorage<Mesh>,
) -> Handle<Mesh> {
    let verts = gen_rectangle_vertices(w, h);
    loader.load_from_data(verts.into(), (), &storage)
}

/// Generate the vertices of a rectangle.
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

/// Generates vertices for a circle. The circle will be made of `resolution`
/// triangles.
pub fn generate_circle_vertices(radius: f32, resolution: usize) -> Vec<PosTex> {
    use std::f32::consts::PI;

    let mut vertices = Vec::with_capacity(resolution * 3);
    let angle_offset = 2.0 * PI / resolution as f32;
    // Helper function to generate the vertex at the specified angle.
    let generate_vertex = |angle: f32| {
        let x = angle.cos();
        let y = angle.sin();
        PosTex {
            position: [x * radius, y * radius, 0.0],
            tex_coord: [x, y],
        }
    };

    for index in 0..resolution {
        vertices.push(PosTex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        });

        vertices.push(generate_vertex(angle_offset * index as f32));
        vertices.push(generate_vertex(angle_offset * (index + 1) as f32));
    }

    vertices
}

// check for removal
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
// check for removal
pub fn window_closed(event: &Event) -> bool{
    match event {
        &Event::WindowEvent { ref event, .. } => match event {
            &WindowEvent::CloseRequested => true,
            _ => false,
        },
        _ => false,
    }
}

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

// TODO: Broken af dependency of TransformBundle pls fix asap lmao
pub fn amethyst_gamedata_base_2d(base: &str) -> Result<GameDataBuilder<'static,'static>>{
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/assets/base/config/display.ron",
        base
    );

    let key_bindings_path = format!(
        "{}/assets/base/config/input.ron",
        base
    );

    GameDataBuilder::default()
        //.with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(
            AnimationBundle::<u32, Material>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            )
        )?
        .with_bundle(AudioBundle::new(|music: &mut Music| music.music.next()))?
        .with(TimedDestroySystem,"timed_destroy", &[])
        .with_basic_renderer(display_config_path, DrawFlat::<PosTex>::new(), false)
}

/*pub fn build_amethyst(game_data_builder: GameDataBuilder<'static,'static>, init_state: State<GameData<'static,'static>>) -> Result<Application<GameData<'static,'static>>>{
    let resources_directory = format!("{}/assets/base", env!("CARGO_MANIFEST_DIR"));
    let game_data = game_data_builder.build()?;
    Application::build(resources_directory, init_state)?.build(game_data)
}*/

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
    fn setup(&mut self, res: &mut amethyst::ecs::Resources){
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

#[derive(Default)]
pub struct NormalOrthoCameraSystem{
    aspect_ratio_cache: f32,
}

impl<'a> System<'a> for NormalOrthoCameraSystem{
    type SystemData = (ReadExpect<'a, ScreenDimensions>, WriteStorage<'a, Camera>);
    fn run(&mut self, (dimensions, mut cameras): Self::SystemData){
        let aspect = dimensions.aspect_ratio();
        println!("Aspect ratio: {}", aspect);
        if aspect != self.aspect_ratio_cache{
            self.aspect_ratio_cache = aspect;

            // If negative, will remove on the x axis instead of stretching the y
            let x_offset = (aspect - 1.0) / 2.0;

            for mut camera in (&mut cameras).join(){
                println!("CHANGING CAM RATIO! {:?}",Ortho{left: -x_offset,right: 1.0 + x_offset,bottom: 0.0,top: 1.0,near: 0.1,far: 2000.0});
                camera.proj = Ortho{left: -x_offset,right: 1.0 + x_offset,bottom: 0.0,top: 1.0,near: 0.1,far: 2000.0}.into();
            }
        }
        
        //random test
        /*for mut camera in (&mut cameras).join(){
            camera.proj = Ortho{left: 0.0,right: 1.0,bottom: -80.0,top: 1.0,near: 0.1,far: 2000.0}.into();
        }*/
    }
}


pub struct UiTimer{
    pub start: f64,
}

impl Component for UiTimer{
    type Storage = VecStorage<Self>;
}

pub struct UiTimerSystem;

impl<'a> System<'a> for UiTimerSystem{
    type SystemData = (ReadStorage<'a,UiTimer>,WriteStorage<'a,UiText>,Read<'a,Time>);
    fn run(&mut self, (timers,mut texts,time): Self::SystemData){
        for (timer,mut text) in (&timers,&mut texts).join(){
            let t = time.absolute_time_seconds() - timer.start;
            text.text = t.to_string(); // Simply show seconds for now.
        }
    }
}


pub trait UiAutoText: Component{
    fn get_text(&self) -> String;
}

pub struct UiAutoTextSystem<T>{
    phantom: PhantomData<T>,
}

impl<'a,T> System<'a> for UiAutoTextSystem<T> where T: Component+UiAutoText{
    type SystemData = (ReadStorage<'a,T>,WriteStorage<'a,UiText>);
    fn run(&mut self, (autotexts,mut texts): Self::SystemData){
        for (autotext,mut text) in (&autotexts,&mut texts).join(){
            text.text = autotext.get_text();
        }
    }
}

pub struct FollowMouse;
impl Component for FollowMouse{
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct FollowMouseSystem<A,B>{
    phantom: PhantomData<(A,B)>,
}

impl<'a,A,B> System<'a> for FollowMouseSystem<A,B> where A: Send+Sync+Hash+Eq+'static+Clone, B: Send+Sync+Hash+Eq+'static+Clone{
    type SystemData = (ReadStorage<'a,FollowMouse>,WriteStorage<'a,Transform>,ReadStorage<'a,GlobalTransform>,ReadExpect<'a,ScreenDimensions>,ReadExpect<'a,InputHandler<A,B>>,ReadStorage<'a,Camera>);
    fn run(&mut self, (follow_mouses,mut transforms, global_transforms, dimension,input,cameras): Self::SystemData){

        fn fancy_normalize(v: f32, a: f32) -> f32 {
            // [0, a]
            // [-1,1]

            v / (0.5 * a) - 1.0
        }

        let width = dimension.width();
        let height = dimension.height();

        if let Some((x,y)) = input.mouse_position() {
            for (gt, cam) in (&global_transforms, &cameras).join() {

                // TODO: Breaks with multiple cameras :ok_hand:
                let proj = cam.proj;
                let view = gt.0;
                let pv = proj * view;
                let inv = pv.invert().expect("Failed to inverse matrix");
                let tmp: Vector4<f32> = [fancy_normalize(x as f32,width),fancy_normalize(y as f32,height),0.0,1.0].into();
                let res = inv * tmp;

                println!("Hopefully mouse pos in world: {:?}",res);


                for (mut transform, _) in (&mut transforms, &follow_mouses).join() {
                    transform.translation = [res.x, res.y, transform.translation.z].into();
                    println!("set pos to {:?}",transform.translation);
                }
            }
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
/*#[cfg(test)]
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
}*/
