use std::{
    io::BufReader,
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

pub struct Client {
    pub id: usize,
    pub connection: TcpStream,
    pub sender: Sender<String>,
    pub messages: Option<Receiver<String>>,
}

impl Client {
    pub fn new(id: usize, c: TcpStream) -> Self {
        let chan = mpsc::channel();
        Self {
            id: id,
            connection: c,
            sender: chan.0,
            messages: Some(chan.1),
        }
    }

    pub fn writer_stream(&self) -> TcpStream {
        self.connection.try_clone().unwrap()
    }

    pub fn create_buf_reader(&self) -> BufReader<TcpStream> {
        BufReader::new(self.connection.try_clone().unwrap())
    }
}
