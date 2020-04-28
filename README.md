# cput
the cput tool works as client and server. it generates UDP traffic and measures UDP packet loss also it measures TCP round-trip time between two end points. the output / statistics can be realtime or json so you can integrate with your script quickly. 
![topo](/docs/imgs/cput_diagram.png?raw=true "cput")

Client Side
```
host1#cput -c --server-addr 192.158.10.23:3055
cput client mode enabled
[1000 UDP pkts sent (localhost), 1000 UDP pkts recvd (192.168.10.23), TCP RTT min/avg/max 12.546/14.256/16.803 ms]
```
Server Side
```
host2#cput -s --server-bind 192.158.10.23:3055
cput server mode enabled
[10000 packets sent (192.158.11.44), 10000 packets received (localhost)]
```

Client Side - Json Pretty
```
host1#cput -c --server-addr 192.158.10.23:3055 --json-pretty
cput client mode enabled
{
  "udp_packets_transmitted": 1000,
  "udp_packets_received": 1000,
  "tcp_round_trip_time": {
    "min": 12.775,
    "max": 20.17,
    "avg": 14.78,
    "count": 10
  }
}
```

```
Circuit Throughput 0.0.1

            host1# cput -s 
            host2# cput -c --server-addr host1:3055
            

USAGE:
    cput [FLAGS] [OPTIONS]

FLAGS:
    -c, --client         enable client mode
    -h, --help           Prints help information
        --json           set json output
        --json-pretty    set json pretty output
    -l, --loopback       enable loopback mode
    -s, --server         enable server mode
    -V, --version        Prints version information

OPTIONS:
    -n, --count <count>                packet limitation
        --mtu <mtu>                    UDP packet / payload size
    -r, --rate-limit <rate>            set rate limit
        --server-addr <server-addr>    server / host ip::port address
        --server-bind <server-bind>    server bind ip::port address
        --source-addr <source-addr>    client source ip::port address
    -t, --thread <thread>              threads number
```
