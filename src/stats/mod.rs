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
struct TcpRtt {
    min: f64,
    max: f64,
    avg: f64,
    count: u64,
}

#[derive(Serialize)]
struct OutputJson<'a> {
    udp_packets_transmitted: u64,
    udp_packets_received: u64,
    tcp_round_trip_time: &'a TcpRtt,
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

    pub fn start(&self) -> Result<(), std::io::Error> {
        let cursor = cursor();
        let total_tx = AtomicU64::new(0);
        let total_rx = Arc::new(AtomicU64::new(0));
        let mut tcp_rtt = TcpRtt::new();
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
                                 let (tb, rtt) = match Self::tcp_client(
                                     &self.tcp_server_addr,
                                     TcpBucket{tx: total_tx.load(Ordering::Relaxed),
                                     rx:0, peer: "".to_string()}
                                    ){
                                        Ok((a,b)) => (a,b),
                                        Err(e) => return Err(e)
                                    };

                                    tcp_rtt.update(rtt);

                                    total_rx.store(tb.rx, Ordering::Relaxed);

                                    // quit signal
                                    Self::tcp_client(
                                     &self.tcp_server_addr,
                                     TcpBucket{tx: 0,
                                     rx:0, peer: "".to_string()}
                                    )?;
                              }

                              self.output(
                                total_tx.load(Ordering::Relaxed),
                                total_rx.load(Ordering::Relaxed),
                                &peer,
                                &peer_server,
                                &tcp_rtt,
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
                            // quit signal
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
                let (tb, rtt) = match Self::tcp_client(
                    &self.tcp_server_addr,
                    TcpBucket {
                        tx: total_tx.load(Ordering::Relaxed),
                        rx: 0,
                        peer: "".to_string(),
                    },
                ) {
                    Ok((a, b)) => (a, b),
                    Err(e) => return Err(e),
                };

                tcp_rtt.update(rtt);

                total_rx.store(tb.rx, Ordering::Relaxed);
                peer_server = tb.peer;

                timer.update();
            }

            self.output(
                total_tx.load(Ordering::Relaxed),
                total_rx.load(Ordering::Relaxed),
                &peer,
                &peer_server,
                &tcp_rtt,
                false,
            );
        }

        cursor.show().unwrap();

        Ok(())
    }

    pub fn _is_server_reachable(&self) -> bool {
        let mut stream = match TcpStream::connect("54.67.98.109:8080") {
            Ok(stream) => stream,
            Err(_) => {
                eprintln!("server side is not reachable");
                return false;
            }
        };

        let b = TcpBucket {
            rx: 0,
            tx: 0,
            peer: "".to_string(),
        };

        let serialized = serde_json::to_string(&b).unwrap();
        stream.write(&serialized.as_bytes()).unwrap();

        true
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

                    match serde_json::to_string(&b) {
                        Ok(serialized) => {
                            let mut a = [0u8; 128];
                            padding(serialized.as_bytes(), &mut a);
                            stream.write(&a).unwrap();
                        }
                        Err(e) => println!("{}", e),
                    };

                    // read

                    let mut buf = vec![];
                    stream.read_to_end(&mut buf).unwrap();
                    match serde_json::from_slice::<TcpBucket>(&buf) {
                        Ok(deserialized) => {
                            tx.send(TcpBucket {
                                tx: deserialized.tx,
                                rx: 0,
                                peer: stream.peer_addr().unwrap().to_string(),
                            })
                            .unwrap();
                        }
                        Err(e) => eprintln!("{}", e),
                    };
                }
                Err(e) => eprintln!("tcp server error: {}", e),
            }
        }
    }

    fn tcp_client(addr: &String, b: TcpBucket) -> Result<(TcpBucket, f64), std::io::Error> {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true).expect("set_nodelay call failed");

        let now = Instant::now();

        // write

        let serialized = serde_json::to_string(&b).unwrap();
        stream.write(serialized.as_bytes()).unwrap();

        // read

        let mut buf = [0u8; 128];
        stream.read_exact(&mut buf).unwrap();

        let rtt = now.elapsed().as_micros();

        let buf_sliced = &buf[..buf[buf.len() - 1] as usize];
        let mut deserialized: TcpBucket = serde_json::from_slice(&buf_sliced).unwrap();

        deserialized.peer = addr.to_string();

        Ok((deserialized, (rtt as f64) / 1000.0))
    }

    fn output(
        &self,
        tx: u64,
        rx: u64,
        peer: &String,
        peer_server: &String,
        tcp_rtt: &TcpRtt,
        last_iter: bool,
    ) {
        match self.output_type {
            Output::Json => {
                if last_iter {
                    let d = OutputJson {
                        udp_packets_transmitted: tx,
                        udp_packets_received: rx,
                        tcp_round_trip_time: tcp_rtt,
                    };

                    let j = serde_json::to_string(&d).unwrap();
                    println!("{}", j);
                }
            }
            Output::JsonPretty => {
                if last_iter {
                    let d = OutputJson {
                        udp_packets_transmitted: tx,
                        udp_packets_received: rx,
                        tcp_round_trip_time: tcp_rtt,
                    };

                    let j = serde_json::to_string_pretty(&d).unwrap();
                    println!("{}", j);
                }
            }
            _ => print!(
                "\r[{} UDP pkts sent ({}), {} UDP pkts recvd ({}), TCP RTT min/avg/max {:.3}/{:.3}/{:.3} ms]",
                tx, peer, rx, peer_server, tcp_rtt.min, tcp_rtt.avg, tcp_rtt.max,
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
        self.time_last.elapsed().as_millis() > 500
    }
}

impl TcpRtt {
    fn new() -> Self {
        TcpRtt {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            count: 0,
        }
    }

    fn update(&mut self, rtt: f64) {
        if self.count == 0 {
            self.min = rtt;
            self.max = rtt;
            self.avg = rtt;
        } else {
            if rtt > self.max {
                self.max = rtt;
            }
            if rtt < self.min {
                self.min = rtt;
            }
            self.avg = (self.avg + rtt) / 2.0;
        }

        self.count += 1;
    }
}

fn padding(a: &[u8], b: &mut [u8]) {
    b[b.len() - 1] = a.len() as u8;
    for (i, x) in a.iter().enumerate() {
        b[i] = *x;
    }
}
