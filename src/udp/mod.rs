use crate::stats::Bucket;
use crossbeam::channel::Sender;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct Client<'a> {
    pub src_addr: &'a str,
    pub dst_addr: &'a str,
    pub thread_num: u8,
    pub count: u128,
    pub rate_limit: u64,
    pub stats_tx: Sender<Bucket>,
}

impl<'a> Client<'a> {
    pub fn new(stats_tx: Sender<Bucket>) -> Self {
        Client {
            src_addr: "0.0.0.0:3056",
            dst_addr: "127.0.0.1:3055",
            thread_num: 4,
            count: 10000,
            rate_limit: 1000,
            stats_tx: stats_tx,
        }
    }

    pub fn start(&self) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind(self.src_addr).expect("couldn't bind the socket");
        socket
            .connect(self.dst_addr)
            .expect("couldn't connect to peer");
        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u128));

        for _ in 0..self.thread_num {
            let socket_cloned = socket.try_clone().expect("couldn't clone the socket");
            let payload = &[0u8; 1500];

            let stats_tx_cloned = self.stats_tx.clone();
            let lock2 = lock.clone();
            let count = self.count;
            let rate_limit = self.rate_limit;
            let thread_num = self.thread_num;

            threads.push(thread::spawn(move || loop {
                let mut guard = lock2.lock().unwrap();
                if *guard >= count {
                    break;
                }
                socket_cloned.send(payload).expect("failed write to server");
                stats_tx_cloned
                    .send(Bucket { tx: 1, rx: 0 })
                    .expect("failed write to stats");
                *guard += 1;
                drop(guard);

                Self::rate_limit(rate_limit, thread_num);
            }));
        }

        for t in threads {
            t.join().unwrap();
        }

        // completed signal
        self.stats_tx.send(Bucket { tx: 0, rx: 0 }).unwrap();

        Ok(())
    }

    fn rate_limit(rate_limit: u64, thread_num: u8) {
        sleep(Duration::from_micros(
            1000000 / (rate_limit / thread_num as u64),
        ));
    }
}

pub struct Server<'a> {
    pub addr: &'a str,
    pub thread_num: u8,
    pub stats_tx: Sender<Bucket>,
}

impl<'a> Server<'a> {
    pub fn new(stats_tx: Sender<Bucket>) -> Self {
        Server {
            addr: "0.0.0.0:3055",
            thread_num: 4,
            stats_tx: stats_tx,
        }
    }

    pub fn start(&self) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind(self.addr)?;
        let mut threads = Vec::new();

        for _ in 0..self.thread_num {
            let socket_cloned = socket.try_clone().expect("couldn't clone the socket");
            let mut buf = [0; 1500];
            let stats_tx_cloned = self.stats_tx.clone();
            threads.push(thread::spawn(move || loop {
                match socket_cloned.recv_from(&mut buf) {
                    Ok((_, _)) => {
                        stats_tx_cloned
                            .send(Bucket { tx: 0, rx: 1 })
                            .expect("couldn't write to channel");
                    }
                    Err(e) => {
                        eprintln!("error: {}", e);
                    }
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }

        Ok(())
    }
}
