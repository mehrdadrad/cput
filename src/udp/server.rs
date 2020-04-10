pub struct Server {}

use std::net::UdpSocket;
use std::str;
use std::thread;

impl Server {
    pub fn run(&self) {
        let socket = UdpSocket::bind("0.0.0.0:3055").expect("couldn't bind the socket");

        loop {
            let mut buf = [0; 1500];
            let socket = socket.try_clone().expect("couldn't clone the socket");
            match socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    thread::spawn(move || {
                        let msg = str::from_utf8(&buf[..n]).unwrap();
                        println!("{:?} {:?}", src, msg);
                    });
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                }
            }
        }
    }
}

pub fn new() -> Server {
    Server {}
}
