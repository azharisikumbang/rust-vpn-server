use std::{io::{BufRead, BufReader}, net::TcpStream};

use crate::client::Client;

pub fn read_message(reader: &mut BufReader<TcpStream>) -> String {
    let mut msg = String::new();
    reader.read_line(&mut msg).unwrap();
    
    msg.trim().to_string()
}

pub fn display_welcome_message(client: &Client) -> () {
    let id = client.id.clone();
    client.sender.send(format!("Your Client ID : {id}")).unwrap()
}

pub fn display_error_message(client: &Client, msg: String) -> () {
    client.sender.send(format!("[Error] {msg}")).unwrap();
}