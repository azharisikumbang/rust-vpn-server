mod client;
mod registry;
mod router;
mod ui;
mod utils;

use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::{client::Client, registry::Registry};

type ClientRegistry = Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>;

fn create_listener(addr: &str) -> TcpListener {
    println!("Server listening at {}", addr);
    TcpListener::bind(addr).unwrap()
}

// fn create_client_registry() -> ClientRegistry {
//     Arc::new(Mutex::new(HashMap::new()))
// }

/*
fn generate_client_id(registry: &ClientRegistry) -> usize {
    let clients = registry.lock().unwrap();
    clients.len() + 1
}

fn handle_incoming_request(_req: Incoming) -> Result<(), Box<dyn Error>> {
    Ok(())
}
*/

// fn cleanup_registry(reg_id: &usize, clients: &ClientRegistry) -> Option<mpsc::Sender<String>> {
//     let mut reg = clients.lock().unwrap();
//     reg.remove(reg_id)
// }

// fn add_sender(tx_id: usize, tx: Sender<String>, client: &ClientRegistry) -> Result<(), String> {
//     client
//         .lock()
//         .map_err(|_| "Error: Failed to lock registry.".to_string())?
//         .insert(tx_id, tx);

//     Ok(())
// }

fn main() {
    let listener: TcpListener = create_listener("127.0.0.1:9999");
    let registry: Registry = Registry::new();
    let mut next_id: usize = 1;

    for stream in listener.incoming() {
        let client_id: usize = next_id;
        next_id += 1;

        let mut client = Client::new(client_id, stream.unwrap());
        registry
            .add_client(&client)
            .inspect_err(|err| ui::display_error_message(&client, err.to_string()))
            .unwrap();

        let registry_cl = registry.clone();
        let writer_stream = client.writer_stream();
        let messages = client.messages.take().expect("s");

        thread::spawn(move || {
            thread::spawn(move || {
                let mut wr = writer_stream;

                for message in messages {
                    if writeln!(wr, "> {message}").is_err() {
                        break;
                    }
                }
            });

            ui::display_welcome_message(&client);
            let mut buf_reader = BufReader::new(client.connection.try_clone().unwrap());

            loop {
                let line = ui::read_message(&mut buf_reader);
                if line.is_empty() {
                    continue;
                }

                let (target_id, sender_msg) = match utils::parse_message(line.as_str()) {
                    Ok((a, b)) => (a, b),
                    Err(err) => {
                        ui::display_error_message(&client, err);
                        continue;
                    }
                };

                if client_id == target_id {
                    ui::display_error_message(&client, "cannot send to your id".to_string());
                    continue;
                }

                router::send_message(target_id, sender_msg.to_string(), &registry_cl.container)
                    .inspect_err(|e| ui::display_error_message(&client, e.to_string()))
                    .unwrap();
            }
            // registry_cl.remove(&client);
            // drop(registry_cl);
        });
    }
}
