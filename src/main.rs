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
                let path_parts = path.split('/').collect::<Vec<&str>>();
                let route = path_parts[1];

                println!("path_parts: {:?}", path_parts);
                println!("Path: {path}");

                let mut response = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();

                match route {
                    "" => {
                        "HTTP/1.1 200 OK\r\n\r\n".clone_into(&mut response);
                    }
                    "echo" => {
                        let value = if path_parts.len() > 2 {
                            path_parts[2]
                        } else {
                            ""
                        };

                        format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", value.len(), value).clone_into(&mut response);
                    }
                    _ => {}
                }

                println!("response: {}", response);

                tcp_stream
                    .write_all(response.as_bytes())
                    .expect("send response");
            }
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }
}
