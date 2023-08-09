use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

pub type ServerFiles = HashMap<String, Vec<u8>>;

#[typestate::typestate]
pub mod worker {
    use super::ServerFiles;
    use std::{collections::VecDeque, io::BufReader, net::TcpStream};

    #[automaton]
    pub struct FileServerWorker {
        pub read_stream: BufReader<TcpStream>,
        pub write_stream: TcpStream,
        pub files: ServerFiles,
    }

    #[state]
    pub struct Idle;
    pub trait Idle {
        fn start(stream: TcpStream, files: ServerFiles) -> Idle;
        fn read_command(self) -> Command;
    }

    #[state]
    pub struct FileRequested {
        pub filename: String,
    }
    pub trait FileRequested {
        fn respond(self) -> Respond;
    }

    #[state]
    pub struct Send {
        pub remaining_bytes: VecDeque<u8>,
    }
    pub trait Send {
        fn send_byte(self) -> Respond;
    }

    #[state]
    pub struct EndResponse;
    pub trait EndResponse {
        fn end_response(self) -> Idle;
    }

    #[state]
    pub struct CloseConnection;
    pub trait CloseConnection {
        fn close_connection(self);
    }

    pub enum Command {
        #[metadata(label = "Requested a file")]
        FileRequested,
        #[metadata(label = "End of request")]
        CloseConnection,
    }

    pub enum Respond {
        #[metadata(label = "File has more bytes to send")]
        Send,
        #[metadata(label = "End of request")]
        EndResponse,
    }
}

pub use worker::*;

impl IdleState for FileServerWorker<Idle> {
    fn start(stream: TcpStream, files: ServerFiles) -> FileServerWorker<Idle> {
        let read_stream = BufReader::new(stream.try_clone().unwrap());
        FileServerWorker {
            read_stream,
            write_stream: stream,
            files,
            state: Idle,
        }
    }

    fn read_command(mut self) -> Command {
        let mut line = String::new();
        self.read_stream.read_line(&mut line).unwrap();
        match line.trim() {
            "REQUEST" => {
                let mut filename = String::new();
                self.read_stream.read_line(&mut filename).unwrap();

                Command::FileRequested(FileServerWorker {
                    read_stream: self.read_stream,
                    write_stream: self.write_stream,
                    files: self.files,
                    state: FileRequested { filename },
                })
            }

            "CLOSE" => Command::CloseConnection(FileServerWorker {
                read_stream: self.read_stream,
                write_stream: self.write_stream,
                files: self.files,
                state: CloseConnection,
            }),

            _ => panic!("Invalid command"),
        }
    }
}

impl FileRequestedState for FileServerWorker<FileRequested> {
    fn respond(self) -> Respond {
        match self.files.get(&self.state.filename) {
            Some(file) => Respond::Send(FileServerWorker {
                read_stream: self.read_stream,
                write_stream: self.write_stream,
                state: Send {
                    remaining_bytes: file.clone().into(),
                },
                files: self.files,
            }),
            None => Respond::EndResponse(FileServerWorker {
                read_stream: self.read_stream,
                write_stream: self.write_stream,
                files: self.files,
                state: EndResponse,
            }),
        }
    }
}

impl SendState for FileServerWorker<Send> {
    fn send_byte(mut self) -> Respond {
        match self.state.remaining_bytes.pop_front() {
            Some(byte) => {
                self.write_stream.write(&[byte]).unwrap();
                Respond::Send(FileServerWorker {
                    read_stream: self.read_stream,
                    write_stream: self.write_stream,
                    files: self.files,
                    state: self.state,
                })
            }
            None => Respond::EndResponse(FileServerWorker {
                read_stream: self.read_stream,
                write_stream: self.write_stream,
                files: self.files,
                state: EndResponse,
            }),
        }
    }
}

impl EndResponseState for FileServerWorker<EndResponse> {
    fn end_response(mut self) -> FileServerWorker<Idle> {
        self.write_stream.write(&[0]).unwrap();
        FileServerWorker {
            read_stream: self.read_stream,
            write_stream: self.write_stream,
            files: self.files,
            state: Idle,
        }
    }
}

impl CloseConnectionState for FileServerWorker<CloseConnection> {
    fn close_connection(self) {}
}
