use std::io::Error;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::time;

pub async fn server() -> Result<(), Error> {
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
                    eprintln!("tcp client write error: {} ", e);
                    return;
                }
            }
        });
    }
}

pub async fn client() -> Result<(), Error> {
    let mut stream = TcpStream::connect("127.0.0.1:3057").await?;
    let mut interval = time::interval(Duration::from_millis(10));

    let msg = [0_u8; 1024];

    loop {
        match stream.write(&msg).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("client error: {}", e);
                stream = TcpStream::connect("127.0.0.1:3057").await?;
            }
        }
        interval.tick().await;
    }
}
