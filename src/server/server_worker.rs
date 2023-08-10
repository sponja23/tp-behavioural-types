use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

#[typestate::typestate]
pub mod worker {
    use std::{io::BufReader, net::TcpStream};

    #[automaton]
    pub struct FileServerWorker {
        pub read_stream: BufReader<TcpStream>,
        pub write_stream: TcpStream,
    }

    #[state]
    pub struct Idle;
    pub trait Idle {
        fn start(stream: TcpStream) -> Idle;
        fn read_command(self) -> Command;
    }

    #[state]
    pub struct AnsweringRequest {
        pub filename: String,
    }
    pub trait AnsweringRequest {
        fn send_byte(self, byte: u8) -> AnsweringRequest;
        fn end_response(self) -> Idle;
    }

    #[state]
    pub struct CloseConnection;
    pub trait CloseConnection {
        fn close_connection(self);
    }

    pub enum Command {
        #[metadata(label = "Requested a file")]
        AnsweringRequest,
        #[metadata(label = "End of request")]
        CloseConnection,
    }
}

pub use worker::*;

//
// State transitions
//

impl IdleState for FileServerWorker<Idle> {
    fn start(stream: TcpStream) -> FileServerWorker<Idle> {
        let read_stream = BufReader::new(stream.try_clone().unwrap());
        FileServerWorker {
            read_stream,
            write_stream: stream,
            state: Idle,
        }
    }

    fn read_command(mut self) -> Command {
        match self.read_line().as_str() {
            "REQUEST" => {
                let filename = self.read_line();

                Command::AnsweringRequest(FileServerWorker {
                    read_stream: self.read_stream,
                    write_stream: self.write_stream,
                    state: AnsweringRequest { filename },
                })
            }

            "CLOSE" => Command::CloseConnection(FileServerWorker {
                read_stream: self.read_stream,
                write_stream: self.write_stream,
                state: CloseConnection,
            }),

            _ => panic!("Invalid command"),
        }
    }
}

impl AnsweringRequestState for FileServerWorker<AnsweringRequest> {
    fn send_byte(mut self, byte: u8) -> FileServerWorker<AnsweringRequest> {
        self.write_byte(byte);
        FileServerWorker {
            read_stream: self.read_stream,
            write_stream: self.write_stream,
            state: self.state,
        }
    }

    fn end_response(mut self) -> FileServerWorker<Idle> {
        self.write_byte(0u8);
        FileServerWorker {
            read_stream: self.read_stream,
            write_stream: self.write_stream,
            state: Idle,
        }
    }
}

impl CloseConnectionState for FileServerWorker<CloseConnection> {
    fn close_connection(self) {}
}

//
// Helper functions
//

impl FileServerWorker<Idle> {
    fn read_line(&mut self) -> String {
        let mut line = String::new();
        self.read_stream
            .read_line(&mut line)
            .expect("Failed to read line");
        line.trim().to_string()
    }
}

impl FileServerWorker<AnsweringRequest> {
    fn write_byte(&mut self, byte: u8) {
        self.write_stream
            .write(&[byte])
            .expect("Failed to send byte");
    }
}
