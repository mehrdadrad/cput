extern crate crossbeam;
use crossbeam::channel::{select, unbounded, Receiver, Sender};
use crossterm::cursor;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub enum Output {
    Json,
    JsonPretty,
    Realtime,
}

pub struct Bucket {
    pub rx: u128,
    pub tx: u128,
}

#[derive(Serialize, Deserialize)]
pub struct TcpBucket {
    tx: u64,
    rx: u64,
    peer: String,
}

struct Timer {
    time_last: Instant,
}

#[derive(Serialize)]
struct OutputJson {
    sent_packet: u64,
    received_packet: u64,
}

pub struct Stats {
    pub rx: Receiver<Bucket>,
    pub tcp_server_bind: String,
    pub tcp_server_addr: String,
    pub output_type: Output,
}

impl Stats {
    pub fn new(rx: Receiver<Bucket>) -> Self {
        Stats {
            rx: rx,
            tcp_server_bind: String::new(),
            tcp_server_addr: String::new(),
            output_type: Output::Realtime,
        }
    }

    pub fn start(&self) {
        let cursor = cursor();
        let total_tx = AtomicU64::new(0);
        let total_rx = Arc::new(AtomicU64::new(0));
        let mut timer = Timer::new();
        let mut peer = String::from("localhost");
        let mut peer_server = String::from("localhost");
        let tcp_server_bind = String::from(&self.tcp_server_bind);

        cursor.hide().unwrap();

        let (tcp_tx, tcp_rx) = unbounded::<TcpBucket>();

        let total_rx_cloned = total_rx.clone();
        if !tcp_server_bind.is_empty() {
            thread::spawn(|| Self::tcp_server(tcp_server_bind, tcp_tx, total_rx_cloned));
        }

        loop {
            select! {
                recv(self.rx) -> msg => {
                    match msg {
                        Ok(x) => {
                           if x.tx == 0 && x.rx == 0 {
                              if !self.tcp_server_addr.is_empty() {
                                 let tb = Self::tcp_client(
                                     &self.tcp_server_addr,
                                     TcpBucket{tx: total_tx.load(Ordering::Relaxed),
                                     rx:0, peer: "".to_string()}
                                    );

                                    total_rx.store(tb.rx, Ordering::Relaxed);

                                    // completed signal
                                    Self::tcp_client(
                                     &self.tcp_server_addr,
                                     TcpBucket{tx: 0,
                                     rx:0, peer: "".to_string()}
                                    );
                              }

                              self.output(
                                total_tx.load(Ordering::Relaxed),
                                total_rx.load(Ordering::Relaxed),
                                &peer,
                                &peer_server,
                                true,
                              );


                              break;
                           }

                           total_tx.fetch_add(x.tx as u64, Ordering::SeqCst);
                           total_rx.fetch_add(x.rx as u64, Ordering::SeqCst);

                        }
                        Err(e) => {eprintln!("{}", e)},
                    }
                },
                recv(tcp_rx) -> msg => {
                    match msg {
                        Ok(x) => {
                            // completed signal
                            if x.tx == 0 {
                                break;
                            }

                            total_tx.store(x.tx, Ordering::Relaxed);
                            peer = x.peer;
                        },
                        Err(_) => {},
                    }
                },
            }

            if timer.ticker() && !self.tcp_server_addr.is_empty() {
                let tb = Self::tcp_client(
                    &self.tcp_server_addr,
                    TcpBucket {
                        tx: total_tx.load(Ordering::Relaxed),
                        rx: 0,
                        peer: "".to_string(),
                    },
                );

                total_rx.store(tb.rx, Ordering::Relaxed);
                peer_server = tb.peer;

                timer.update();
            }

            self.output(
                total_tx.load(Ordering::Relaxed),
                total_rx.load(Ordering::Relaxed),
                &peer,
                &peer_server,
                false,
            );
        }
    }

    fn tcp_server(addr: String, tx: Sender<TcpBucket>, total_rx: Arc<AtomicU64>) {
        let listener = TcpListener::bind(addr).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    stream.set_nodelay(true).expect("set_nodelay call failed");

                    // write

                    let b = TcpBucket {
                        rx: total_rx.load(Ordering::Relaxed),
                        tx: 0,
                        peer: "".to_string(),
                    };

                    let serialized = serde_json::to_string(&b).unwrap();
                    let mut a = [0u8; 128];
                    padding(serialized.as_bytes(), &mut a);
                    stream.write(&a).unwrap();

                    // read

                    let mut buf = vec![];
                    stream.read_to_end(&mut buf).unwrap();
                    let deserialized: TcpBucket = serde_json::from_slice(&buf).unwrap();
                    tx.send(TcpBucket {
                        tx: deserialized.tx,
                        rx: 0,
                        peer: stream.peer_addr().unwrap().to_string(),
                    })
                    .unwrap();
                }
                Err(e) => eprintln!("tcp server error: {}", e),
            }
        }
    }

    fn tcp_client(addr: &String, b: TcpBucket) -> TcpBucket {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_nodelay(true).expect("set_nodelay call failed");

        let now = Instant::now();

        // write

        let serialized = serde_json::to_string(&b).unwrap();
        stream.write(serialized.as_bytes()).unwrap();

        // read

        let mut buf = [0u8; 128];
        stream.read_exact(&mut buf).unwrap();

        let rtt = now.elapsed().as_millis();

        let buf_sliced = &buf[..buf[buf.len() - 1] as usize];
        let mut deserialized: TcpBucket = serde_json::from_slice(&buf_sliced).unwrap();

        deserialized.peer = addr.to_string();

        deserialized
    }

    fn output(&self, tx: u64, rx: u64, peer: &String, peer_server: &String, last_iter: bool) {
        match self.output_type {
            Output::Json => {
                if last_iter {
                    let d = OutputJson {
                        sent_packet: tx,
                        received_packet: rx,
                    };

                    let j = serde_json::to_string(&d).unwrap();
                    println!("{}", j);
                }
            }
            Output::JsonPretty => {
                if last_iter {
                    let d = OutputJson {
                        sent_packet: tx,
                        received_packet: rx,
                    };

                    let j = serde_json::to_string_pretty(&d).unwrap();
                    println!("{}", j);
                }
            }
            _ => eprint!(
                "\r[{} packets sent ({}), {} packets received ({})]",
                tx, peer, rx, peer_server,
            ),
        }
    }
}

impl Timer {
    fn new() -> Self {
        Timer {
            time_last: Instant::now(),
        }
    }
    fn update(&mut self) {
        self.time_last = Instant::now();
    }
    fn ticker(&self) -> bool {
        self.time_last.elapsed().as_secs() > 1
    }
}

fn padding(a: &[u8], b: &mut [u8]) {
    b[b.len() - 1] = a.len() as u8;
    for (i, x) in a.iter().enumerate() {
        b[i] = *x;
    }
}
