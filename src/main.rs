mod utils;

use crate::utils::{extract_headers, get_path};
use flate2::bufread::GzEncoder;
use flate2::Compression;
use std::cmp::PartialEq;
use std::fs::read;
use std::io::{BufRead, BufReader, Read};
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
            let basic = http_request[0].split(' ').collect::<Vec<&str>>();
            let method = Method::from(basic[0]);
            let path = basic[1];
            let path_parts = path.split('/').collect::<Vec<&str>>();
            let route = path_parts[1];
            let headers = extract_headers(&http_request);
            let mut response_query = b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec();

            match method {
                Method::Get => match route {
                    "" => {
                        response_query = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
                    }
                    "echo" => {
                        let value = if path_parts.len() > 2 {
                            path_parts[2].to_owned()
                        } else {
                            "".to_owned()
                        };
                        let mut body = Vec::new();

                        body.extend_from_slice(value.as_bytes());

                        let mut content_encoded = "".to_owned();
                        let accept_encoding = headers.get("accept-encoding");

                        if let Some(encoding) = accept_encoding {
                            let encoding = encoding.split(", ").collect::<Vec<&str>>();
                            let accepted = encoding.into_iter().find(|s| s.contains("gzip"));

                            if let Some(s) = accepted {
                                content_encoded = format!("\r\nContent-Encoding: {}", s);
                                body.clear();

                                let val = BufReader::new(value.as_bytes());
                                let mut gz = GzEncoder::new(val, Compression::fast());
                                let _ = gz.read_to_end(&mut body);
                            }
                        }

                        let response = format!(
                            "HTTP/1.1 200 OK{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                            content_encoded,
                            body.len()
                        );

                        response_query = response.as_bytes().to_vec();
                        response_query.extend_from_slice(body.as_slice());
                    }
                    "user-agent" => {
                        let user_agent = headers.get("user-agent");

                        if let Some(value) = user_agent {
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                value.len(),
                                value
                            );
                            response_query = response.as_bytes().to_vec();
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
                                response_query = response.as_bytes().to_vec();
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

                            if content_length.is_some() {
                                let content = http_request.last().unwrap();
                                let mut file = File::create(path).await.expect("Create the file");

                                file.write_all(content.as_bytes())
                                    .await
                                    .expect("Write the data to the file");

                                response_query = b"HTTP/1.1 201 Created\r\n\r\n".to_vec();
                            }
                        }
                    }
                    _ => {}
                },
                Method::Invalid => {}
            }

            stream
                .write_all(response_query.as_slice())
                .await
                .expect("Send response")
        });
    }
}
