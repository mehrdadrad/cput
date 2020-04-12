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
    pub mtu: u16,
}

impl<'a> Client<'a> {
    pub fn new() -> Client<'a> {
        Client {
            src_addr: "0.0.0.0:3056",
            dst_addr: "127.0.0.1:3055",
            thread_num: 4,
            rate: 1000,
            count: 1000,
            mtu: 1500,
        }
    }
    pub fn run(&self) {
        let socket = UdpSocket::bind(self.src_addr).expect("couldn't bind the socket");
        socket
            .connect(self.dst_addr)
            .expect("couldn't connect to peer");

        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u64));

        for _ in 0..self.thread_num {
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

pub struct Server<'a> {
    pub addr: &'a str,
    pub thread_num: u8,
}

impl<'a> Server<'a> {
    pub fn new() -> Server<'a> {
        Server {
            addr: "0.0.0.0:3055",
            thread_num: 4,
        }
    }
    pub fn run(&self) {
        let socket = UdpSocket::bind(self.addr).expect("couldn't bind the socket");

        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u64));

        for _ in 0..self.thread_num {
            let socket = socket.try_clone().expect("couldn't clone the socket");
            let mut buf = [0; 1500];
            let lock2 = lock.clone();

            threads.push(thread::spawn(move || loop {
                match socket.recv_from(&mut buf) {
                    Ok((_, _)) => {
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
