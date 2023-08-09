use server_worker::*;

use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream, ToSocketAddrs},
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

    pub fn run_worker(&self, stream: TcpStream) {
        // Files are cloned because we couldn't figure out how to use lifetime
        // parameters with typestate.
        let mut worker = FileServerWorker::start(stream, self.files.clone());

        loop {
            todo!();
        }
    }

    pub fn start(&self) {
        thread::scope(|s| {
            for stream in self.socket.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        s.spawn(|| self.run_worker(stream));
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        });
    }
}

fn main() {
    let mut files: ServerFiles = HashMap::new();
    files.insert("file-a.txt".into(), "FILE A CONTENTS".into());

    let server = FileServer::new("0.0.0.0:1234", files);
    server.start();
}
