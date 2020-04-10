use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct Client<'a> {
    pub src_addr: &'a str,
    pub dst_addr: &'a str,
    pub thread_num: u8,
    pub rate: u64,
    pub count: u64,
}

impl<'a> Client<'a> {
    pub fn run(&self) {
        let socket = UdpSocket::bind(self.src_addr).expect("couldn't bind the socket");
        socket
            .connect(self.dst_addr)
            .expect("couldn't connect to peer");

        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u64));

        for i in 0..self.thread_num {
            let socket = socket.try_clone().expect("couldn't clone the socket");
            let payload = &[0u8; 1500];
            let wait = self.wait();
            let count = self.count;
            let lock2 = lock.clone();

            threads.push(thread::spawn(move || loop {
                let mut guard = lock2.lock().unwrap();
                if *guard >= count {
                    break;
                }
                socket.send(payload).expect("failed write to server");
                *guard += 1;
                println!("sent! {} {}", i, guard);
                drop(guard);
                sleep(wait);
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }

    fn wait(&self) -> Duration {
        Duration::from_micros(1000000 / (self.rate / self.thread_num as u64))
    }
}

pub fn new<'a>() -> Client<'a> {
    Client {
        src_addr: "0.0.0.0:3056",
        dst_addr: "127.0.0.1:3055",
        thread_num: 4,
        rate: 200,
        count: 100,
    }
}
