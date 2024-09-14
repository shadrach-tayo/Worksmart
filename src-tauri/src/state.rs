#[derive(Default, Debug)]
pub struct AppState {
    pub key_notifier: Option<tokio::sync::broadcast::Sender<()>>,
    pub mouseclick_notifier: Option<tokio::sync::broadcast::Sender<()>>,
    // pub session_shutdown_tx:
}
