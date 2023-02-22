use std::{sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}, error::Error};

use lazy_static::lazy_static;
use discord_rich_presence::{DiscordIpcClient, DiscordIpc, activity::Activity};

const OPENT5_DISCORD_RPC_CLIENT_ID: &str = "1078061707345272902";

lazy_static! {
    static ref CLIENT: Arc<RwLock<DiscordIpcClient>> = Arc::new(RwLock::new(DiscordIpcClient::new(OPENT5_DISCORD_RPC_CLIENT_ID).unwrap()));
    static ref INITED: AtomicBool = AtomicBool::new(false);
}   

pub fn set_activity(activity: Activity) -> Result<(), Box<dyn Error>> {
    let inited = INITED.load(Ordering::Relaxed);
    let lock = CLIENT.clone();
        let mut client = lock.write().unwrap();
    if !inited {
        client.connect()?;
    }

    client.set_activity(activity)
}