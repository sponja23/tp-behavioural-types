use crate::server_worker::FileServerWorker;

use self::server_worker::ServerFiles;

use std::{
    collections::HashMap,
    net::{TcpListener, ToSocketAddrs},
    thread,
};

mod server_worker;

pub struct FileServer {
    socket: TcpListener,
    files: ServerFiles,
}

impl FileServer {
    pub fn new(addr: impl ToSocketAddrs, files: ServerFiles) -> FileServer {
        let socket = TcpListener::bind(addr).expect("Failed to bind to port");
        FileServer { socket, files }
    }

    pub fn start(&self) {
        thread::scope(|s| {
            for stream in self.socket.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        s.spawn(|| {
                            let mut worker = FileServerWorker::new(stream, &self.files);
                            worker.start();
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        })
    }
}

fn main() {
    let mut files: ServerFiles = HashMap::new();
    files.insert("file-a.txt".into(), "FILE A CONTENTS".into());

    let server = FileServer::new("0.0.0.0:1234", files);
    server.start();
}
