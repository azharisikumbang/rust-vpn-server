mod utils;
mod router;

use std::{
    collections::HashMap, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex, mpsc::{self, Sender}}, thread
};

type ClientRegistry = Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>;

fn create_listener(addr: &str) -> TcpListener {
    println!("Server listening at {}", addr);
    TcpListener::bind(addr).unwrap()
}

fn create_client_registry() -> ClientRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

/*
fn generate_client_id(registry: &ClientRegistry) -> usize {
    let clients = registry.lock().unwrap();
    clients.len() + 1
}

fn handle_incoming_request(_req: Incoming) -> Result<(), Box<dyn Error>> {
    Ok(())
}
*/

fn cleanup_registry(reg_id: &usize, clients: &ClientRegistry) -> Option<mpsc::Sender<String>> {
    let mut reg = clients.lock().unwrap();
    reg.remove(reg_id)
}

fn add_sender(tx_id: usize, tx: Sender<String>, client: &ClientRegistry) -> Result<(), String> {
    client
        .lock()
        .map_err(|_| "Error: Failed to lock registry.".to_string())?
        .insert(tx_id, tx);
    
    Ok(())
}

fn main() {
    let listener = create_listener("127.0.0.1:9999");
    let clients = create_client_registry();
    let mut next_id = 1;

    for stream in listener.incoming() {
        let client: TcpStream = stream.unwrap();
        let (own_tx, recv) = mpsc::channel::<String>();
        let client_id = next_id;
        next_id += 1;

        let _ = add_sender(client_id, own_tx.clone(), &clients)
            .inspect_err(|e| eprintln!("{e}"));

        let clients_c = Arc::clone(&clients);
        let writer_stream = client.try_clone().unwrap();

        thread::spawn(move || {
            thread::spawn(move || {
                let mut wr = writer_stream;

                for message in recv {
                    if writeln!(wr, "> {message}").is_err() {
                        break;
                    }
                }
            });

            own_tx.send(format!("Your Client ID : {client_id}")).unwrap();

            let mut buf_reader = BufReader::new(client);

            loop {
                let mut line = String::new();
                match buf_reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("reader error {}", e);
                        break;
                    }
                }

                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                let (target_id, sender_msg) = match utils::parse_message(line) {
                    Ok((a, b)) => (a,b),
                    Err(err ) => {
                        own_tx.send(format!("[Warn] {err}")).unwrap();
                        continue;
                    }
                };

                if client_id == target_id {
                    own_tx.send(format!("[Warn] cannot send to your id.")).unwrap();
                    continue;
                }

                router::send_message(target_id, sender_msg.to_string(), &clients_c)
                    .inspect_err(|e| own_tx.send(e.to_string()).unwrap()).unwrap();
            }

            cleanup_registry(&client_id, &clients_c);
            drop(own_tx);
        });
    }
}
