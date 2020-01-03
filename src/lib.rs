extern crate amethyst;
#[macro_use]
extern crate serde;
extern crate ron;
extern crate serde_json;
#[macro_use]
extern crate log;
pub extern crate crossterm;
pub extern crate dirty;
extern crate fern;
pub extern crate partial_function;
pub extern crate rand;
extern crate roman;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_new;
extern crate discord_rpc_client;
pub extern crate hyper;
pub extern crate hyper_tls;
pub extern crate specs_physics as nphysics_ecs;
extern crate tokio;
extern crate tokio_executor;

mod asset_loader;
mod auth;
mod auto_save;
mod auto_text;
mod follow_mouse;
mod movement;
mod noclip;
mod relative_timer;
mod terminal;
mod time_control;
mod ui_timer;

pub use self::asset_loader::*;
pub use self::auth::*;
pub use self::auto_save::*;
pub use self::auto_text::*;
pub use self::follow_mouse::*;
pub use self::movement::*;
pub use self::noclip::*;
pub use self::relative_timer::*;
pub use self::terminal::*;
pub use self::time_control::*;
pub use self::ui_timer::*;

use amethyst::core::math::{Point3, Vector3};
use amethyst::gltf::VerticeData;

use amethyst::ecs::*;

use amethyst::prelude::*;

use amethyst::utils::removal::Removal;

use discord_rpc_client::Client as DiscordClient;
use hyper::client::HttpConnector;
use hyper::{Body, Chunk, Client, Request, Response};
use hyper_tls::HttpsConnector;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::fmt::Debug;
use std::ops::{Add, Sub};

use std::sync::{Arc, Mutex};

use tokio::prelude::{Future, Stream};
use tokio::runtime::Runtime;

//use nphysics::{World, Body3d};

/*pub trait AssetToFormat<T> where T: Sized{
    fn get_format() -> Format<T>;
}

impl AssetToFormat<Mesh> for Mesh{
    fn get_format() -> Format<Mesh>{
        ObjFormat
    }
}*/

pub fn verts_from_mesh_data(mesh_data: &VerticeData, scale: &Vector3<f32>) -> Vec<Point3<f32>> {
        mesh_data
            .vertices
            .iter()
            .map(|sep| {
                Point3::new(
                    sep[0] * scale.x,
                    sep[1] * scale.y,
                    sep[2] * scale.z,
                )
            })
            .collect::<Vec<_>>()
}

pub fn avg_float_to_string(value: f32, decimals: u32) -> String {
    let mult = 10.0_f32.powf(decimals as f32);
    ((value * mult).ceil() / mult).to_string()
}

// TODO: remove once merged in amethyst
pub fn add_removal_to_entity<T: PartialEq + Clone + Debug + Send + Sync + 'static>(
    entity: Entity,
    id: T,
    world: &mut World,
) {
    world.write_storage::<Removal<T>>().insert(entity, Removal::new(id)).expect(&format!(
        "Failed to insert removalid to entity {:?}.",
        entity
    ));
}

pub fn value_near<B: Add<Output = B> + Sub<Output = B> + PartialOrd + Copy>(
    number: B,
    target: B,
    margin: B,
) -> bool {
    number >= target - margin && number <= target + margin
}

/*pub struct NavigationButton{
    pub target: fn() -> Trans,
}

impl Component for NavigationButton{
    type Storage = VecStorage<Self>;
}*/

/// Discord Rich Presence wrapper around discord_rpc_client
/// Currently errors are not exposed by the library, so I use the log crate
/// to display errors and only return Result<T, ()> from the methods.
///
/// Make sure to properly create your app here: https://discordapp.com/developers/applications
///
/// Usage:
/// ```rs
/// fn init_discord_rich_presence() -> Result<DiscordRichPresence,()> {
///     DiscordRichPresence::new(498979571933380609, "Main Menu", Some("large_image"), Some("Hoppin World"), None, None);
/// }
/// ```
pub struct DiscordRichPresence {
    pub rpc: Arc<Mutex<DiscordClient>>,
    state: String,
    large_image: Option<String>,
    large_image_text: Option<String>,
    small_image: Option<String>,
    small_image_text: Option<String>,
}

impl DiscordRichPresence {
    pub fn new(
        app_id: u64,
        state: String,
        large_image: Option<String>,
        large_image_text: Option<String>,
        small_image: Option<String>,
        small_image_text: Option<String>,
    ) -> std::result::Result<Self, ()> {
        let mut rpc = DiscordClient::new(app_id);
        if let Err(e) = rpc {
            error!("Failed to create discord rich presence client: {:?}", e);
            return Err(());
        }
        rpc.as_mut().unwrap().start();
        let mut drp = DiscordRichPresence {
            rpc: Arc::new(Mutex::new(rpc.unwrap())),
            state,
            large_image,
            large_image_text,
            small_image,
            small_image_text,
        };
        drp.update();
        Ok(drp)
    }
    pub fn set_state(&mut self, state: String) {
        self.state = state;
        self.update();
    }

    pub fn update(&mut self) {
        if let Err(e) = self.rpc.lock().unwrap().set_activity(|a| {
            a.state(self.state.clone()).assets(|ass| {
                let mut tmp = ass;
                if let Some(ref t) = self.large_image {
                    tmp = tmp.large_image(t.clone());
                }
                if let Some(ref t) = self.large_image_text {
                    tmp = tmp.large_text(t.clone());
                }
                if let Some(ref t) = self.small_image {
                    tmp = tmp.small_image(t.clone());
                }
                if let Some(ref t) = self.small_image_text {
                    tmp = tmp.small_text(t.clone());
                }
                tmp
            })
        }) {
            error!("Failed to set discord rich presence state: {}", e);
        }
    }
}

impl Drop for DiscordRichPresence {
    fn drop(&mut self) {
        if let Err(e) = self.rpc.lock().unwrap().clear_activity() {
            eprintln!("Failed to clear discord rich presence activity {:?}", e);
        }
    }
}

/// Changes the discord rich presence state, if present in the world.
pub fn set_discord_state(state: String, world: &mut World) {
    if let Some(mut drp) = world.try_fetch_mut::<DiscordRichPresence>() {
        drp.set_state(state);
    }
}

pub fn https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::new(2).expect("TLS initialization failed");
    Client::builder().build::<_, hyper::Body>(https)
}

pub fn post_json(url: String, data: String) -> Request<Body> {
    Request::post(&url)
        .header("Content-Type", "application/json")
        .body(Body::from(data))
        .unwrap()
}

pub fn post_json_typed<T: Serialize>(url: String, data: T) -> Request<Body> {
    Request::post(&url)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&data).expect(
            "Failed to serialize data to json for post request creation.",
        )))
        .expect("Failed to create post `Request`")
}

pub fn exec_http_request(
    client: &Client<HttpsConnector<HttpConnector>, Body>,
    request: Request<Body>,
    future_runtime: &mut Runtime,
    callback_queue: &CallbackQueue,
    on_success: Box<dyn Fn(Response<Body>) -> Box<dyn Fn(&mut World) + Send> + Send>,
    on_error: Box<dyn Fn(hyper::Error) -> Box<dyn Fn(&mut World) + Send> + Send>,
) {
    let send_handle1 = callback_queue.send_handle();
    let send_handle2 = callback_queue.send_handle();
    let future = client
        .request(request)
        /*.and_then(move |result| {
            println!("Response: {}", result.status());
            println!("Headers: {:#?}", result.headers());

            // The body is a stream, and for_each returns a new Future
            // when the stream is finished, and calls the closure on
            // each chunk of the body...
            result.into_body().for_each(move |chunk| {
                /*io::stdout().write_all(&chunk)
                    .map_err(|e| panic!("example expects stdout is open, error={}", e))*/
                match serde_json::from_slice::<Auth>(&chunk) {
                    Ok(a) => {},
                    Err(e) => eprintln!("Failed to parse received data to Auth: {}", e),
                }
                Ok(())
            })
            //serde_json::from_slice::<Auth>(result.into_body())
        })*/
        // If all good, just tell the user...
        .map(move |result| {
            let callback = on_success(result);
            send_handle1.send(callback).expect("Failed to send Callback to CallbackQueue from future completion.");
        })
        // If there was an error, let the user know...
        .map_err(move |err| {
            let callback = on_error(err);
            send_handle2.send(callback).expect("Failed to send Callback to CallbackQueue from future error.");
        });

    future_runtime.spawn(future);
}

/// Warning: Blocks the thread in which it is called until the stream has been fully consumed.
/// Avoid using with file downloads.
/// This will only return the first parse error instead of all of them, because its easier to use that way.
pub fn response_to_chunks(
    response: Response<Body>,
) -> Vec<std::result::Result<Chunk, hyper::Error>> {
    response.into_body().wait().collect::<Vec<_>>()
}

pub fn parse_chunk<T: DeserializeOwned>(
    chunk: &Chunk,
) -> std::result::Result<T, serde_json::Error> {
    serde_json::from_slice::<T>(&chunk)
}

pub fn sec_to_display(secs: f64, decimals: usize) -> String {
    if secs > -0.00001 && secs < 0.00001 {
        String::from("-")
    } else {
        format!("{:0.*}", decimals, secs)
    }
}

// Building parts + logic

/*
Mock data

Player Stats:
Health
Physical Damage
Mana

Player Skills:
Repulsive Orb (-10 mana/throw, 1 sec cooldown)

Player Levels:
global level
health level
mana regen level
physical damage level

Item Def:
Mana pendant (+ 10 max mana, +1.0 mana regen / sec)

Item Instance:
Greater Mana pendant (Capacity Enchant):
    Mana pendant regular +
    Greater (20% buff all effects) (+2 max mana, +0.2 mana regen)
    Capacity Enchant (+20 max mana)

*/

/*
  2D controllers
*/
