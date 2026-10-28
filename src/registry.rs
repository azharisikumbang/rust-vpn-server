use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::Sender},
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

    // pub fn create_client(&self, stream: TcpStream) -> Result<Client, String> {
    //     let id = match self.container.lock() {
    //         Ok(val) => {
    //             let exp_id = val.len() + 1;
    //             if val.get(&exp_id).is_none() {
    //                 exp_id
    //             } else {
    //                 exp_id + 1
    //             }
    //         }
    //         Err(_) => 0,
    //     };

    //     if id <= 0 {
    //         return Err("Error: failed to create new client".to_string());
    //     }

    //     Ok(Client::new(id, stream))
    // }

    pub fn add_client(&self, client: &Client) -> Result<(), String> {
        self.container
            .lock()
            .map_err(|_| "Error: Failed to lock registry.".to_string())?
            .insert(client.id, client.sender.clone());

        Ok(())
    }

    pub fn remove(&self, client: &Client) -> () {
        self.container.lock().unwrap().remove(&client.id);
    }
}
