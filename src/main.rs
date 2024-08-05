use std::io::Write;
use std::net::TcpListener;

fn main() {
    let ip = "127.0.0.1";
    let port = "4221";

    let listener = TcpListener::bind(format!("{}:{}", ip, port)).unwrap();

    println!("Http server started on: {}:{}", ip, port);

    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp_stream) => {
                println!("accepted new connection");
                let response = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();
                tcp_stream.write_all(response).expect("send response 200");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
