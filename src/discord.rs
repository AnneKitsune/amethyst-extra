use amethyst::ecs::World;
use discord_rpc_client::Client as DiscordClient;
use std::sync::mpsc::*;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

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
    pub rpc: DiscordClient,
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
    ) -> Self {
        let rpc = DiscordClient::new(app_id);
        DiscordRichPresence {
            rpc,
            state,
            large_image,
            large_image_text,
            small_image,
            small_image_text,
        }
    }

    pub fn start(&mut self) {
        self.rpc.start();
        self.update();
    }

    pub fn set_state(&mut self, state: String) {
        self.state = state;
        self.update();
    }

    pub fn update(&mut self) {
        let state = self.state.clone();
        let large_image = self.large_image.clone();
        let large_image_text = self.large_image_text.clone();
        let small_image = self.small_image.clone();
        let small_image_text = self.small_image_text.clone();
        if let Err(e) = self.rpc.set_activity(|a| {
            a.state(state).assets(|ass| {
                let mut tmp = ass;
                if let Some(t) = large_image {
                    tmp = tmp.large_image(t);
                }
                if let Some(t) = large_image_text {
                    tmp = tmp.large_text(t);
                }
                if let Some(t) = small_image {
                    tmp = tmp.small_image(t);
                }
                if let Some(t) = small_image_text {
                    tmp = tmp.small_text(t);
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
        if let Err(e) = self.rpc.clear_activity() {
            eprintln!("Failed to clear discord rich presence activity {:?}", e);
        }
    }
}

pub struct DiscordThreadHolder {
    pub thread: JoinHandle<()>,
    pub sender: Mutex<Sender<DiscordThreadMessage>>,
}

impl DiscordThreadHolder {
    pub fn new(mut presence: DiscordRichPresence) -> Self {
        let (tx, rx) = channel();
        let thread = thread::spawn(move || {
            presence.start();
            loop {
                match rx.recv() {
                    Ok(DiscordThreadMessage::Update) => presence.update(),
                    Ok(DiscordThreadMessage::SetState(state)) => presence.set_state(state),
                    Err(_) => return,
                }
            }
        });
        Self {
            thread,
            sender: Mutex::new(tx),
        }
    }
}

pub enum DiscordThreadMessage {
    Update, SetState(String),
}

/// Changes the discord rich presence state, if present in the world.
pub fn set_discord_state(state: String, world: &mut World) {
    world.fetch_mut::<DiscordThreadHolder>().sender.lock().expect("Failed to acquire mutex lock for DiscordThreadHolder sender handle").send(DiscordThreadMessage::SetState(state)).expect("Failed to send state update message through DiscordThreadHolder's sender");
}
