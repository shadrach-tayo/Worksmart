use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

#[derive(Default, Debug)]
pub struct AppState {
    pub keystroke_rx: Option<KeystrokeBroadCaster>,
    pub mouseclick_rx: Option<MouseclickBroadCaster>,
    // pub session_shutdown_tx:
}

pub type MouseclickBroadCaster = broadcast::Sender<DateTime<Utc>>;
pub type KeystrokeBroadCaster = broadcast::Sender<DateTime<Utc>>;
