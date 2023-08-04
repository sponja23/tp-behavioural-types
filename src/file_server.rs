use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
};

pub struct FileServer {
    socket: TcpListener,
}

impl FileServer {
    pub fn new(address: impl ToSocketAddrs) -> FileServer {
        let socket = TcpListener::bind(address).expect("Failed to bind to port");
        FileServer { socket }
    }

    pub fn start(&self) {
        for stream in self.socket.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(|| {
                        let mut worker = FileServerWorker::new(stream);
                        worker.start();
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}

pub struct FileServerWorker {
    stream: BufReader<TcpStream>,
}

impl FileServerWorker {
    pub fn new(stream: TcpStream) -> FileServerWorker {
        FileServerWorker {
            stream: BufReader::new(stream),
        }
    }

    pub fn read_command(&mut self) -> String {
        let mut buf = String::new();
        self.stream
            .read_line(&mut buf)
            .expect("Failed to read from stream");
        buf
    }

    pub fn start(&mut self) {
        loop {
            match self.read_command().trim() {
                "REQUEST" => {
                    todo!("handle request")
                }
                "CLOSE" => break,
                _ => {
                    println!("Invalid command")
                }
            }
        }
    }
}
