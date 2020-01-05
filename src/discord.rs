use amethyst::ecs::World;
use discord_rpc_client::Client as DiscordClient;
use std::sync::{Arc, Mutex};

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
        let rpc = DiscordClient::new(app_id);
        let drp = DiscordRichPresence {
            rpc: Arc::new(Mutex::new(rpc)),
            state,
            large_image,
            large_image_text,
            small_image,
            small_image_text,
        };
        Ok(drp)
    }

    pub fn start(&mut self) {
        self.rpc.lock().unwrap().start();
        self.update();
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
