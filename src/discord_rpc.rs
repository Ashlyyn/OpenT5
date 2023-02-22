use std::{sync::{Arc, RwLock, atomic::{AtomicBool, Ordering, AtomicI64}}, error::Error, time::UNIX_EPOCH};

use lazy_static::lazy_static;
use discord_rich_presence::{DiscordIpcClient, DiscordIpc, activity::{Activity, Assets, Timestamps}};

const OPENT5_DISCORD_RPC_CLIENT_ID: &str = "1078061707345272902";
const OPENT5_DISCORD_RPC_ICON_URL: &str = "https://cdn.discordapp.com/app-icons/1078061707345272902/82f49e326e037b523c65ca1c451a916e.png?size=256";

lazy_static! {
    static ref CLIENT: Arc<RwLock<DiscordIpcClient>> = Arc::new(RwLock::new(DiscordIpcClient::new(OPENT5_DISCORD_RPC_CLIENT_ID).unwrap()));
    static ref INITED: AtomicBool = AtomicBool::new(false);
    static ref START_TIME: AtomicI64 = AtomicI64::new(0);
}

fn start_time() -> i64 {
    START_TIME.load(Ordering::Relaxed) as _
}

fn set_start_time(time: i64) {
    START_TIME.store(time, Ordering::Relaxed);
}

pub fn init() -> Result<(), Box<dyn Error>> {
    let lock = CLIENT.clone();
    let mut client = lock.write().unwrap();

    client.connect()?;

    // Can't use sys::milliseconds here, because sys::milliseconds tracks the 
    // timespan since the program was launched instead of since the Unix Epoch.
    // If we set the start time as sys::milliseconds, Discord simply won't 
    // display the application's rich presence.
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    set_start_time(now);

    client.set_activity(
        Activity::new()
            .assets(Assets::new().small_image(OPENT5_DISCORD_RPC_ICON_URL))
            .timestamps(Timestamps::new().start(start_time()))
    )
}

pub fn set_activity(activity: Activity) -> Result<(), Box<dyn Error>> {
    let inited = INITED.load(Ordering::Relaxed);

    if !inited {
        init()?;
    }

    let lock = CLIENT.clone();
    let mut client = lock.write().unwrap();
    
    // Always add the icon and start time to whatever other activity info
    // was passed, overwriting if necessary
    client.set_activity(
        activity
            .assets(Assets::new().small_image(OPENT5_DISCORD_RPC_ICON_URL))
            .timestamps(Timestamps::new().start(start_time()))
    )
}