mod client;
mod registry;
mod router;
mod ui;
mod utils;

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
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

fn handle_client(mut client: Client, registry: &Registry) -> Result<(), String> {
    if registry.add_client(&client).is_err() {
        return Err("Error: Failed to add client to registry.".to_string());
    }

    let stream_writer = client.writer_stream();
    let messages = client.messages.take().unwrap();
    let registry_cl = registry.clone();

    thread::spawn(move || {
        thread::spawn(move || {
            let mut wr = stream_writer;

            for msg in messages {
                if writeln!(wr, "{msg}").is_err() {
                    break;
                }
            }
        });

        ui::display_welcome_message(&client);

        let mut buf_reader = client.create_buf_reader();
        loop {
            let mut msg = String::new();
            buf_reader.read_line(&mut msg).unwrap();
            msg = msg.trim().to_string();

            if msg.is_empty() {
                continue;
            }

            let (target_id, message) = match utils::parse_message(&msg) {
                Ok((a, b)) => (a, b),
                Err(e) => {
                    ui::display_error_message(&client, e);
                    continue;
                }
            };

            if client.id == target_id {
                ui::display_error_message(
                    &client,
                    "Warning: you cannot sending message to your own id".to_string(),
                );
                continue;
            }

            router::send_message(target_id, message.to_string(), &registry_cl.container)
                .inspect_err(|e| ui::display_error_message(&client, e.to_string()))
                .unwrap();
        }
    });

    // registry.remove(client.id.clone());
    // drop(client);

    Ok(())
}

fn main() {
    let listener: TcpListener = create_listener("127.0.0.1:9999");
    let registry: Registry = Registry::new();
    let mut next_id: usize = 1;

    for stream in listener.incoming() {
        let client_id: usize = next_id;
        next_id += 1;

        let client = Client::new(client_id, stream.unwrap());
        if handle_client(client, &registry).is_err() {
            // registry.remove(&client);
            // drop(client);
            break;
        }
    }
}
