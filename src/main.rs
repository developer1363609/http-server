use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use http_server::connection::tcp::TcpServerConnection;
use http_server::http::server::Server;


fn main() {
    let args:Vec<String> = env::args().collect();
    let socket = &args[1];
    let tcp_server_connection = TcpServerConnection::new(
        SocketAddr::from_str(socket).expect("Specified socket does not exist")
    ).expect("Unable to initialize connection. Server shutdown");
    let http_server = Server::new(tcp_server_connection);
    http_server.run();
}
