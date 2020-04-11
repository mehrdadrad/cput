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
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .help("bind address and port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .help("host address and port")
                .takes_value(true),
        )
        .get_matches();

    if matches.is_present("server") == true {
        let mut s = udp::server::new();

        if let Some(x) = matches.value_of("bind") {
            s.addr = x;
        }

        if let Some(x) = matches.value_of("thread") {
            s.thread_num = x.parse::<u8>().unwrap();
        }

        s.run();
    } else {
        let mut c = udp::client::new();

        if let Some(x) = matches.value_of("count") {
            c.count = x.parse::<u64>().unwrap();
        }

        if let Some(x) = matches.value_of("rate") {
            c.rate = x.parse::<u64>().unwrap();
        }

        if let Some(x) = matches.value_of("thread") {
            c.thread_num = x.parse::<u8>().unwrap();
        }

        if let Some(x) = matches.value_of("bind") {
            c.src_addr = x;
        }

        if let Some(x) = matches.value_of("host") {
            c.dst_addr = x;
        }

        c.run();
    }
}
