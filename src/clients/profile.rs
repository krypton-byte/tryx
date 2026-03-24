use std::sync::Arc;

use pyo3::pyclass;
use tokio::sync::watch;
use whatsapp_rust::Client;

#[pyclass]
pub struct ProfileClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ProfileClient {
    pub fn new(client_rx: watch::Receiver<Option<Arc<Client>>>) -> Self {
        Self { client_rx }
    }
    pub fn get_client(&self) -> Arc<Client> {
        self.client_rx
            .borrow()
            .clone()
            .expect("Bot is not running")
    }
}