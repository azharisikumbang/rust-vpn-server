use crate::ClientRegistry;

pub fn send_message(
    target_id: usize,
    message: String,
    clients: &ClientRegistry
) -> Result<(), String> {

    let target = clients
        .lock()
        .map_err(|_| "Error: Failed to lock registry.".to_string())?
        .get(&target_id)
        .cloned()
        .ok_or_else(|| {
            format!("Error: Failed to send message, target {target_id} not found.")
        })?;

    target
        .send(message)
        .map_err(|_| {
            format!("Error: Failed to send message to {target_id}.")
        })?;
   
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::{Arc, Mutex, mpsc}};
    use super::*;

    #[test]
    fn test_send_message_to_valid_target_id() {
        let message= String::from("some message");
        let clients:Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (target,recv ) = mpsc::channel::<String>();

        clients.lock().unwrap().insert(1, target);

        let result = send_message(1, message.clone(), &clients);
        assert!(result.is_ok());

        let res_msg = recv.recv().unwrap();
        assert_eq!(res_msg, message);
    }

    #[test]
    fn test_send_message_to_non_exists_id() {
        let message = String::from("hello");
        let clients: Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));

        let result = send_message(1, message, &clients);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error: Failed to send message, target 1 not found.")

    }

    #[test]
    fn test_send_message_to_disconnected_receiver() {
        let message= String::from("some message");
        let clients:Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (target, recv ) = mpsc::channel::<String>();

        clients.lock().unwrap().insert(1, target);
        drop(recv);

        let res = send_message(1, message, &clients);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Error: Failed to send message to 1.");
    }
}