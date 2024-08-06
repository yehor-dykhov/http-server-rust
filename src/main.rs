use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::str;

fn main() {
    let ip = "127.0.0.1";
    let port = "4221";

    let listener = TcpListener::bind(format!("{}:{}", ip, port)).expect("Run TCP server");

    println!("Server run on: {ip}:{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp_stream) => {
                let buf_reader = BufReader::new(&mut tcp_stream);
                let http_request: Vec<_> = buf_reader
                    .lines()
                    .map(|result| result.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();

                let basic = http_request[0].split(' ').collect::<Vec<&str>>();
                let path = basic[1];

                println!("Path: {path}");

                let mut response = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();

                if path == "/" {
                    "HTTP/1.1 200 OK\r\n\r\n".clone_into(&mut response);
                }

                tcp_stream
                    .write_all(response.as_bytes())
                    .expect("send response");
            }
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }
}
