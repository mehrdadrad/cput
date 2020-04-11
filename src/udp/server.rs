pub struct Server<'a> {
    pub addr: &'a str,
    pub thread_num: u8,
}

use std::net::UdpSocket;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

impl<'a> Server<'a> {
    pub fn run(&self) {
        let socket = UdpSocket::bind(self.addr).expect("couldn't bind the socket");

        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u64));

        for _ in 0..2 {
            let socket = socket.try_clone().expect("couldn't clone the socket");
            let mut buf = [0; 1500];
            let lock2 = lock.clone();

            threads.push(thread::spawn(move || loop {
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let _msg = str::from_utf8(&buf[..n]).unwrap();
                        let mut guard = lock2.lock().unwrap();
                        *guard += 1;
                    }
                    Err(e) => {
                        eprintln!("error: {}", e);
                    }
                }
            }));
        }

        let lock2 = lock.clone();
        let mut counter = 0;
        thread::spawn(move || loop {
            sleep(Duration::from_secs(5));
            let guard = lock2.lock().unwrap();
            if *guard > 0 && *guard > counter {
                println!(
                    "packets rcvd: {} rate: {} pps",
                    *guard,
                    (*guard - counter) / 5
                );
            }
            counter = *guard;
        });

        for t in threads {
            t.join().unwrap();
        }
    }
}

pub fn new<'a>() -> Server<'a> {
    Server {
        addr: "0.0.0.0:3055",
        thread_num: 4,
    }
}
