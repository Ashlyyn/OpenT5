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

    set_start_time(std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as _);

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
    
    client.set_activity(
        activity
            .assets(Assets::new().small_image(OPENT5_DISCORD_RPC_ICON_URL))
            .timestamps(Timestamps::new().start(start_time()))
    )
}