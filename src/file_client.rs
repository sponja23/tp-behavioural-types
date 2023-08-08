use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

pub struct FileClient {
    stream: TcpStream,
}

impl FileClient {
    pub fn new(addr: impl ToSocketAddrs) -> FileClient {
        FileClient {
            stream: TcpStream::connect(addr).expect("Connection failed"),
        }
    }

    pub fn request_file(&mut self, filename: String) -> Vec<u8> {
        self.stream
            .write_all(format!("REQUEST\n{filename}\n").as_bytes())
            .expect("Request failed");

        todo!()
    }

    pub fn close(self) {}
}
