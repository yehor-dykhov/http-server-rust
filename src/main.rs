mod utils;

use std::cmp::PartialEq;
use std::fs::read;
use std::io::BufRead;
use std::{env, str};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use crate::utils::{extract_headers, get_path};

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
            let headers = extract_headers(&http_request);
            println!("HEADERS: {:?}", &headers);

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

                        let accept_encoding = headers.get("accept-encoding");

                        let content_encoded = if accept_encoding.is_some() && accept_encoding.unwrap().eq("gzip") {
                            format!("\r\nContent-Encoding: {}", accept_encoding.unwrap())
                        } else {
                            "".to_owned()
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            content_encoded,
                            value.len(),
                            value
                        );
                        response.clone_into(&mut response_query);
                    }
                    "user-agent" => {
                        let user_agent = headers.get("user-agent");

                        if let Some(value) = user_agent {
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
                            let content_length = headers.get("content-length");

                            if let Some(len_data) = content_length {
                                let len = len_data.parse::<u32>().unwrap();
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
