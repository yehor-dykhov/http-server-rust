use std::io::BufRead;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let ip = "127.0.0.1";
    let port = "4221";

    let listener = TcpListener::bind(format!("{}:{}", ip, port))
        .await
        .expect("Run TCP server");

    println!("Server run on: {ip}:{port}");

    loop {
        let (mut stream, _) = listener.accept().await.expect("Listening...");

        tokio::spawn(async move {
            let mut buf: [u8; 1024] = [0; 1024];
            let size = stream.read(&mut buf).await.expect("Read to buffer");

            let http_request: Vec<_> = buf[..size]
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let basic = http_request[0].split(' ').collect::<Vec<&str>>();
            let path = basic[1];
            let path_parts = path.split('/').collect::<Vec<&str>>();
            let route = path_parts[1];

            println!("http_request: {:?}", http_request);
            println!("path_parts: {:?}", path_parts);
            println!("Path: {path}");

            let mut response_query = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();

            match route {
                "" => {
                    "HTTP/1.1 200 OK\r\n\r\n".clone_into(&mut response_query);
                }
                "echo" => {
                    let value = if path_parts.len() > 2 {
                        path_parts[2]
                    } else {
                        ""
                    };

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        value.len(),
                        value
                    );
                    response.clone_into(&mut response_query);
                }
                "user-agent" => {
                    let user_agent = http_request.iter().find(|s| s.contains("User-Agent"));

                    if let Some(ua) = user_agent {
                        let ua_data = ua.split(' ').collect::<Vec<&str>>();
                        let value = ua_data[1];
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", value.len(), value);
                        response.clone_into(&mut response_query)
                    }
                }
                _ => {}
            }

            println!("response: {}", response_query);

            stream
                .write_all(response_query.as_bytes())
                .await
                .expect("Send response")
        });
    }
}
