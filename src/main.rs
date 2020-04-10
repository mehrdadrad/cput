extern crate clap;
use clap::{App, Arg};
mod udp;

fn main() {
    let matches = App::new("circuit throughput check")
        .version("0.0.1")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("enable server mode"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("count numbers")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rate")
                .short("r")
                .long("rate")
                .help("rate of packets per second")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("thread")
                .short("t")
                .long("thread")
                .help("threads number")
                .takes_value(true),
        )
        .get_matches();

    if matches.is_present("server") == true {
        let s = udp::server::new();
        s.run();
    } else {
        let mut c = udp::client::new();
        match matches.value_of("count") {
            Some(x) => c.count = x.parse::<u64>().unwrap(),
            None => {}
        }
        match matches.value_of("rate") {
            Some(x) => c.rate = x.parse::<u64>().unwrap(),
            None => {}
        }
        match matches.value_of("thread") {
            Some(x) => c.thread_num = x.parse::<u8>().unwrap(),
            None => {}
        }

        c.run();
    }
}
