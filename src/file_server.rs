use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
};

pub type ServerFiles = HashMap<String, Vec<u8>>;

pub struct FileServer {
    socket: TcpListener,
    files: ServerFiles,
}

impl FileServer {
    pub fn new(address: impl ToSocketAddrs, files: ServerFiles) -> FileServer {
        let socket = TcpListener::bind(address).expect("Failed to bind to port");
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

pub struct FileServerWorker<'a> {
    stream: TcpStream,
    files: &'a ServerFiles,
}

impl<'a> FileServerWorker<'a> {
    pub fn new(stream: TcpStream, files: &'a ServerFiles) -> FileServerWorker {
        FileServerWorker { stream, files }
    }

    fn read_line(&mut self) -> String {
        let mut buf = String::new();
        let mut reader = BufReader::new(&mut self.stream);
        reader
            .read_line(&mut buf)
            .expect("Failed to read from stream");
        buf
    }

    fn send_byte(&mut self, value: u8) {
        self.stream.write(&[value]).expect("Failed to send byte");
    }

    fn handle_request(&mut self, filename: String) {
        let file_contents = self.files.get(&filename);
        match file_contents {
            Some(contents) => {
                for byte in contents {
                    self.send_byte(*byte)
                }
                self.send_byte(0)
            }
            None => self.send_byte(0),
        }
    }

    pub fn start(&mut self) {
        loop {
            match self.read_line().trim() {
                "REQUEST" => {
                    let filename = self.read_line();
                    self.handle_request(filename)
                }
                "CLOSE" => break,
                _ => {
                    println!("Invalid command")
                }
            }
        }
    }
}
