use server_worker::*;

use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
};

mod server_worker;

type ServerFiles = HashMap<String, Vec<u8>>;

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
        let worker = FileServerWorker::start(stream);

        let mut command = worker.read_command();

        loop {
            command = match command {
                Command::AnsweringRequest(mut worker) => {
                    let filename = worker.state.filename.clone();
                    let file = self.files.get(&filename);
                    match file {
                        Some(file) => {
                            for byte in file {
                                worker = worker.send_byte(*byte);
                            }
                            worker.end_response().read_command()
                        }
                        None => {
                            println!("File not found: {}", filename);
                            worker.end_response().read_command()
                        }
                    }
                }
                Command::CloseConnection(worker) => {
                    worker.close_connection();
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
