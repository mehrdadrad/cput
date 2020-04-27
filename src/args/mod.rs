use crate::{stats, udp};
use clap::{App, Arg};
use std::num::ParseIntError;

pub struct Cmd {}

impl Cmd {
    pub fn new() -> Self {
        Cmd {}
    }

    pub fn init(
        &self,
        client: &mut udp::Client,
        server: &mut udp::Server,
        stats: &mut stats::Stats,
    ) -> Result<usize, ParseIntError> {
        let matches = App::new("Circuit Throughput")
            .version("0.0.1")
            .about(
                r#"
            host1# cput -s 
            host2# cput -c --server-addr host1:3055
            "#,
            )
            .arg(
                Arg::with_name("server")
                    .short("s")
                    .long("server")
                    .conflicts_with("client")
                    .help("enable server mode"),
            )
            .arg(
                Arg::with_name("client")
                    .short("c")
                    .long("client")
                    .conflicts_with("server")
                    .help("enable client mode"),
            )
            .arg(
                Arg::with_name("loopback")
                    .short("l")
                    .long("loopback")
                    .conflicts_with("server")
                    .conflicts_with("client")
                    .help("enable loopback mode"),
            )
            .arg(Arg::with_name("json").long("json").help("set json output"))
            .arg(
                Arg::with_name("json-pretty")
                    .long("json-pretty")
                    .help("set json pretty output"),
            )
            .arg(
                Arg::with_name("thread")
                    .short("t")
                    .long("thread")
                    .help("threads number")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("count")
                    .short("n")
                    .long("count")
                    .help("packet limitation")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("rate")
                    .short("r")
                    .long("rate-limit")
                    .help("set rate limit")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("source-addr")
                    .long("source-addr")
                    .help("client source ip::port address")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("server-addr")
                    .long("server-addr")
                    .help("server / host ip::port address")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("server-bind")
                    .long("server-bind")
                    .help("server bind ip::port address")
                    .takes_value(true),
            )
            .get_matches();

        if let Some(x) = matches.value_of("thread") {
            client.thread_num = x.parse::<u8>()?;
            server.thread_num = x.parse::<u8>()?;
        }

        if let Some(x) = matches.value_of("count") {
            client.count = x.parse::<u128>()?;
        }

        if let Some(x) = matches.value_of("source-addr") {
            client.src_addr = x.to_string();
        }

        if let Some(x) = matches.value_of("server-addr") {
            client.dst_addr = x.to_string();
        }

        if let Some(x) = matches.value_of("server-bind") {
            server.addr = x.to_string();
        }

        if let Some(x) = matches.value_of("rate") {
            client.rate_limit = x.parse::<u64>()?;
        }

        if matches.is_present("json") {
            stats.output_type = stats::Output::Json;
        }

        if matches.is_present("json-pretty") {
            stats.output_type = stats::Output::JsonPretty;
        }

        if matches.is_present("server") == true {
            stats.tcp_server_addr = String::from("0.0.0.0:8080");
            return Ok(0);
        }

        if matches.is_present("client") == true {
            stats.tcp_client_addr = String::from("127.0.0.1:8080");
            return Ok(1);
        }

        Ok(2)
    }
}
