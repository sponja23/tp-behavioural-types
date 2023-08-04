mod file_server;

fn main() {
    let server = file_server::FileServer::new("0.0.0.0:1234");
    server.start();
}
