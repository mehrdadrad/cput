use crossbeam::channel::unbounded;
use std::num::ParseIntError;
use std::thread;
mod args;
mod stats;
mod udp;

fn main() -> Result<(), ParseIntError> {
    let (stats_tx, stats_rx) = unbounded::<stats::Bucket>();
    let cmd = args::Cmd::new();

    let mut client = udp::Client::new(stats_tx.clone());
    let mut server = udp::Server::new(stats_tx.clone());
    let mut stats = stats::Stats::new(stats_rx);

    let mode = cmd.init(&mut client, &mut server, &mut stats)?;

    match mode {
        0 => {
            println!("cput server mode enabled");
            thread::spawn(move || server.start());
        }
        1 => {
            println!("cput client mode enabled");
            thread::spawn(move || client.start());
        }
        _ => {
            println!("cput loopback mode enabled");
            thread::spawn(move || server.start());
            thread::spawn(move || client.start());
        }
    };

    stats.start().unwrap();

    Ok(())
}
