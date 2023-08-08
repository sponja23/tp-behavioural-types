use std::collections::HashMap;

use file_server::ServerFiles;

mod file_client;
mod file_server;

fn main() {
    let mut files: ServerFiles = HashMap::new();
    files.insert("file-a.txt".into(), "FILE A CONTENTS".into());

    let server = file_server::FileServer::new("0.0.0.0:1234", files);
    server.start();
}
