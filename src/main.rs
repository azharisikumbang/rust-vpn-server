use std::{
    collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex, mpsc}, thread
};

type ClientRegistry = Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>;

fn create_listener(addr: &str) -> TcpListener {
    println!("Server listening at {}", addr);
    TcpListener::bind(addr).unwrap()
}

fn create_client_registry() -> ClientRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn generate_client_id(registry: &ClientRegistry) -> usize {
    let clients = registry.lock().unwrap();
    clients.len() + 1
}

fn main() {
    let listener = create_listener("127.0.0.1:9999");
    let clients = create_client_registry();
    let mut next_id = 1;


    for stream in listener.incoming() {
        let client: TcpStream = match stream {
            Ok(val) => val,
            Err(e) => {
                eprintln!("failed to connect: {}", e);
                continue;
            }
        };

        let (own_tx, recv) = mpsc::channel::<String>();
        let client_id = next_id;
        next_id += 1;
        
        {
            let mut registry = clients.lock().unwrap();
            registry.insert(client_id, own_tx.clone());
        }

        let clients_c = Arc::clone(&clients);
        let writer_stream = client.try_clone().unwrap();

        thread::spawn(move || {
            
            thread::spawn(move || {
                let mut wr = writer_stream;

                for message in recv {
                    if writeln!(wr, "{message}").is_err() {
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

                let Some((target, sender_msg)) = line.split_once(' ') else {
                    own_tx.send(format!("Invalid format {client_id}")).unwrap();
                    continue;
                };

                
                let target_id: usize = match target.parse::<usize>() {
                    Ok(val) => val,
                    Err(_) => {
                        own_tx.send(format!("Invalid client id")).unwrap();
                        continue;
                    }
                };

                let target_tx = match clients_c.lock() {
                    Ok(tgt) => tgt.get(&target_id).cloned(),
                    Err(err) => {
                        eprintln!("error: {err}");
                        break;
                    },
                };

                match target_tx { 
                    Some(t) => {
                        if t.send(format!("From {client_id}: {sender_msg}")).is_err() {
                            own_tx.send(format!("Failed to send to id : {target_id}")).unwrap();
                        }
                    },
                    None => {
                        own_tx.send(format!("(None) Failed to send to id : {target_id}")).unwrap();
                    }
                }
            }

            match clients_c.lock() {
                Ok(mut registry) => {
                    println!("registry {client_id} removed");
                    registry.remove(&client_id);
                },
                Err(e) => {
                    eprintln!("error removing client {client_id} : {e}");
                }
            }

            drop(own_tx);
        });
    }
}
