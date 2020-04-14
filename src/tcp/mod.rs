#![allow(dead_code)]

use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::task;

pub struct Server {}
pub struct Client {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    #[tokio::main]
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut listener = TcpListener::bind("127.0.0.1:3057").await?;
        loop {
            let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    let n = match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    if let Err(e) = socket.write_all(&buf[0..n]).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            });
        }
    }
}

impl Client {
    pub fn new() -> Client {
        Client {}
    }

    #[tokio::main]
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut stream = TcpStream::connect("127.0.0.1:3057").await?;
            tokio::spawn(async move {
                match stream.write(b"cput cput cput").await {
                    Ok(_) => println!("write"),
                    Err(e) => eprintln!("{}", e),
                }

                drop(stream);
            });

            task::spawn_blocking(|| std::thread::sleep(Duration::from_millis(50))).await?;
        }
    }
}
