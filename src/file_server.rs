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
    read_stream: BufReader<TcpStream>,
    write_stream: TcpStream,
    files: &'a ServerFiles,
}

impl<'a> FileServerWorker<'a> {
    pub fn new(stream: TcpStream, files: &'a ServerFiles) -> FileServerWorker {
        FileServerWorker {
            read_stream: BufReader::new(stream.try_clone().expect("Failed to clone TcpStream")),
            write_stream: stream,
            files,
        }
    }

    fn read_line(&mut self) -> String {
        let mut buf = String::new();
        self.read_stream
            .read_line(&mut buf)
            .expect("Failed to read from stream");
        buf.trim().into()
    }

    fn send_byte(&mut self, value: u8) {
        self.write_stream
            .write_all(&[value])
            .expect("Failed to send byte");
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
            match self.read_line().as_str() {
                "REQUEST" => {
                    let filename = self.read_line();
                    println!("Requested filename: {filename}");
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
