pub struct Shutdown {
    shutdown: bool,
    notify: tokio::sync::broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: tokio::sync::broadcast::Receiver<()>) -> Self {
        Self {
            shutdown: false,
            notify,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.recv().await;

        self.shutdown = true;
    }
}
