use std::net::{TcpListener, ToSocketAddrs};

pub struct FileServer {
    socket: TcpListener,
}

impl FileServer {
    pub fn new(address: impl ToSocketAddrs) -> FileServer {
        let socket = TcpListener::bind(address).unwrap();
        FileServer { socket }
    }

    pub fn start(&self) {
        for stream in self.socket.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}
