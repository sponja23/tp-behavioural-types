use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

#[typestate::typestate]
pub mod client {
    use std::net::{TcpStream, ToSocketAddrs};

    #[automaton]
    pub struct FileClient {
        pub stream: TcpStream,
    }

    #[state]
    pub struct Idle;
    pub trait Idle {
        fn connect_to(addr: impl ToSocketAddrs) -> Idle;
        fn start_request(self, filename: String) -> AwaitingResponse;
        fn close_connection(self);
    }

    #[state]
    pub struct ResponseEnded;
    pub trait ResponseEnded {
        fn end_request(self) -> Idle;
    }

    #[state]
    pub struct ReceivingData;
    pub trait ReceivingData {
        fn receive_byte(self, buf: &mut [u8; 1]) -> AwaitingResponse;
    }

    pub enum AwaitingResponse {
        #[metadata(label = "Receiving a byte")]
        ReceivingData,
        #[metadata(label = "End of response")]
        ResponseEnded,
    }
}

use client::*;

//
// State transitions
//

impl IdleState for FileClient<Idle> {
    fn connect_to(addr: impl ToSocketAddrs) -> FileClient<Idle> {
        let stream = TcpStream::connect(addr).expect("Connection failed");

        FileClient {
            stream,
            state: Idle,
        }
    }

    fn start_request(mut self, filename: String) -> AwaitingResponse {
        self.send(format!("REQUEST\n{filename}\n").as_bytes());

        AwaitingResponse::ReceivingData(FileClient {
            stream: self.stream,
            state: ReceivingData,
        })
    }

    fn close_connection(mut self) {
        self.send(b"CLOSE\n");
    }
}

impl ReceivingDataState for FileClient<ReceivingData> {
    fn receive_byte(mut self, buf: &mut [u8; 1]) -> AwaitingResponse {
        let byte = self.read_byte();

        buf[0] = byte;

        match byte {
            0 => AwaitingResponse::ResponseEnded(FileClient {
                stream: self.stream,
                state: ResponseEnded,
            }),
            _ => AwaitingResponse::ReceivingData(FileClient {
                stream: self.stream,
                state: ReceivingData,
            }),
        }
    }
}

impl ResponseEndedState for FileClient<ResponseEnded> {
    fn end_request(self) -> FileClient<Idle> {
        FileClient {
            stream: self.stream,
            state: Idle,
        }
    }
}

//
// Auxiliary functions
//

impl FileClient<Idle> {
    // Send a byte array to the server
    // Can only be done in Idle state, to initiate a request or to close the connection
    fn send(&mut self, data: &[u8]) {
        self.stream
            .write_all(data)
            .expect("Failed to write to stream");
    }

    // Request a file from the server
    // Can only be done in Idle state, to initiate a request
    fn request_file(self, filename: String, buf: &mut Vec<u8>) -> FileClient<Idle> {
        let mut response = self.start_request(filename);

        loop {
            response = match response {
                AwaitingResponse::ReceivingData(worker) => {
                    // Read a byte from the server
                    let mut byte = [0; 1];
                    let response = worker.receive_byte(&mut byte);
                    buf.push(byte[0]);
                    response
                }
                AwaitingResponse::ResponseEnded(worker) => {
                    return worker.end_request();
                }
            }
        }
    }
}

impl FileClient<ReceivingData> {
    // Read a single byte from the server
    // Can only be done in ReceivingByte state, to receive a byte of the file
    fn read_byte(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.stream
            .read_exact(&mut buf)
            .expect("Failed to read from stream");
        buf[0]
    }
}

fn main() {
    let mut client = FileClient::connect_to("0.0.0.0:1234");

    let mut buf = Vec::new();
    client = client.request_file("file-a.txt".to_string(), &mut buf);

    println!("{}", String::from_utf8(buf).unwrap());

    client.close_connection();
}
