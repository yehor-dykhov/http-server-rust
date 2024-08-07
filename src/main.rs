use itertools::Itertools;
use std::cmp::PartialEq;
use std::fs::read;
use std::io::BufRead;
use std::{env, str};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(PartialEq)]
enum Method {
    Get,
    Post,
    Invalid,
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "get" => Method::Get,
            "post" => Method::Post,
            _ => Method::Invalid,
        }
    }
}

fn get_path(path_parts: Vec<&str>, args: Vec<String>) -> Option<String> {
    let file_name = if path_parts.len() > 2 {
        path_parts[2]
    } else {
        ""
    };

    let file_path =
        if let Some((index, _)) = &args.iter().find_position(|arg| arg.contains("--directory")) {
            &args[index + 1]
        } else {
            ""
        };

    if file_name.is_empty() || file_path.is_empty() {
        return None;
    }

    Some(format!("{}/{}", file_path, file_name))
}

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

            let http_request: Vec<_> = buf[..size].lines().map(|result| result.unwrap()).collect();

            println!("request: {:?}", http_request);

            let basic = http_request[0].split(' ').collect::<Vec<&str>>();
            let method = Method::from(basic[0]);
            let path = basic[1];
            let path_parts = path.split('/').collect::<Vec<&str>>();
            let route = path_parts[1];

            let mut response_query = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();

            match method {
                Method::Get => match route {
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
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                value.len(),
                                value
                            );
                            response.clone_into(&mut response_query)
                        }
                    }
                    "files" => {
                        let args: Vec<String> = env::args().collect();

                        let file_path = get_path(path_parts, args);

                        if let Some(path) = file_path {
                            if let Ok(data) = read(path) {
                                let txt: &str = str::from_utf8(&data).expect("Convert to string");

                                let response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                                    txt.len(),
                                    txt
                                );
                                response.clone_into(&mut response_query);
                            }
                        }
                    }
                    _ => {}
                },
                Method::Post => match route {
                    "files" => {
                        let args: Vec<String> = env::args().collect();

                        let file_path = get_path(path_parts, args);

                        if let Some(path) = file_path {
                            let content_length =
                                http_request.iter().find(|s| s.contains("Content-Length"));

                            if let Some(len_data) = content_length {
                                let len = len_data.split(' ').collect::<Vec<&str>>()[1];
                                let content = http_request.last().unwrap();
                                println!("len: {}", len);

                                let mut file = File::create(path).await.expect("Create the file");
                                file.write_all(content.as_bytes())
                                    .await
                                    .expect("Write the data to the file");

                                let response = "HTTP/1.1 201 Created\r\n\r\n";
                                response.clone_into(&mut response_query);
                            }
                        }
                    }
                    _ => {}
                },
                Method::Invalid => {}
            }

            println!("response: {}", response_query);

            stream
                .write_all(response_query.as_bytes())
                .await
                .expect("Send response")
        });
    }
}
