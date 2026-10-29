use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::Sender},
    usize,
};

use crate::client::Client;

#[derive(Clone)]
pub struct Registry {
    pub container: Arc<Mutex<HashMap<usize, Sender<String>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            container: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_client(&self, client: &Client) -> Result<(), String> {
        self.container
            .lock()
            .map_err(|_| "Error: Failed to lock registry.".to_string())?
            .insert(client.id, client.sender.clone());

        Ok(())
    }

    pub fn remove(&self, id: usize) -> () {
        self.container.lock().unwrap().remove(&id);
    }
}
