use crate::stats::Bucket;
use crossbeam::channel::Sender;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct Client {
    pub src_addr: String,
    pub dst_addr: String,
    pub thread_num: u8,
    pub mtu: usize,
    pub count: u128,
    pub rate_limit: u64,
    pub stats_tx: Sender<Bucket>,
}

impl Client {
    pub fn new(stats_tx: Sender<Bucket>) -> Self {
        Client {
            src_addr: "0.0.0.0:3056".to_string(),
            dst_addr: "127.0.0.1:3055".to_string(),
            thread_num: 4,
            mtu: 512,
            count: 10000,
            rate_limit: 1000,
            stats_tx: stats_tx,
        }
    }

    pub fn start(&self) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind(&self.src_addr).expect("couldn't bind the socket");
        socket
            .connect(&self.dst_addr)
            .expect("couldn't connect to peer");
        let mut threads = Vec::new();
        let lock = Arc::new(Mutex::new(0_u128));

        for _ in 0..self.thread_num {
            let socket_cloned = socket.try_clone().expect("couldn't clone the socket");
            let payload = vec![0u8; self.mtu];

            let stats_tx_cloned = self.stats_tx.clone();
            let lock2 = lock.clone();
            let count = self.count;
            let rate_limit = self.rate_limit;
            let thread_num = self.thread_num;

            threads.push(thread::spawn(move || -> Result<(), std::io::Error> {
                loop {
                    let mut guard = lock2.lock().unwrap();
                    if *guard >= count {
                        break;
                    }
                    socket_cloned.send(&payload)?;

                    stats_tx_cloned
                        .send(Bucket { tx: 1, rx: 0 })
                        .expect("failed write to stats");
                    *guard += 1;
                    drop(guard);

                    Self::rate_limit(rate_limit, thread_num);
                }

                Ok(())
            }));
        }

        for t in threads {
            if let Err(e) = t.join().unwrap() {
                self.stats_quit_sig();
                return Err(e);
            }
        }

        self.stats_quit_sig();

        Ok(())
    }

    fn stats_quit_sig(&self) {
        self.stats_tx.send(Bucket { tx: 0, rx: 0 }).unwrap();
    }

    fn rate_limit(rate_limit: u64, thread_num: u8) {
        sleep(Duration::from_micros(
            1000000 / (rate_limit / thread_num as u64),
        ));
    }
}

pub struct Server {
    pub addr: String,
    pub thread_num: u8,
    pub stats_tx: Sender<Bucket>,
}

impl Server {
    pub fn new(stats_tx: Sender<Bucket>) -> Self {
        Server {
            addr: "0.0.0.0:3055".to_string(),
            thread_num: 4,
            stats_tx: stats_tx,
        }
    }

    pub fn start(&self) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind(&self.addr)?;
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
