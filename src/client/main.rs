#![allow(dead_code)]

use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

pub struct FileClient {
    stream: TcpStream,
}

impl FileClient {
    pub fn connect_to(addr: impl ToSocketAddrs) -> FileClient {
        FileClient {
            stream: TcpStream::connect(addr).expect("Connection failed"),
        }
    }

    fn send(&mut self, data: &[u8]) {
        self.stream
            .write_all(data)
            .expect("Failed to write to stream");
    }

    fn read_byte(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.stream
            .read_exact(&mut buf)
            .expect("Failed to read from stream");
        buf[0]
    }

    pub fn request_file(&mut self, filename: String) -> Vec<u8> {
        self.send(format!("REQUEST\n{filename}\n").as_bytes());

        let mut file_bytes = Vec::new();

        loop {
            let byte = self.read_byte();

            if byte == 0 {
                break;
            }

            file_bytes.push(byte);
        }

        file_bytes
    }

    pub fn close(mut self) {
        self.send(b"CLOSE\n");
    }
}

fn main() {
    let mut client = FileClient::connect_to("0.0.0.0:1234");

    let file_bytes = client.request_file("file-a.txt".into());
    println!("{}", String::from_utf8(file_bytes).unwrap());
    client.close();
}
