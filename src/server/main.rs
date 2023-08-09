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
        // File map is cloned because we couldn't figure out how to use
        // lifetime parameters with the typestate macro.
        let mut worker = FileServerWorker::start(stream, self.files.clone());

        loop {
            match worker.read_command() {
                Command::FileRequested(request_worker) => {
                    let mut response = request_worker.respond();

                    loop {
                        match response {
                            Respond::Send(response_worker) => {
                                response = response_worker.send_byte();
                            }
                            Respond::EndResponse(response_worker) => {
                                worker = response_worker.end_response();
                                break;
                            }
                        }
                    }
                }
                Command::CloseConnection(closing_worker) => {
                    closing_worker.close_connection();
                    break;
                }
            }
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
