# cput
circuit throughput 

Client Side
```
host1#cput -c --server-addr 192.158.10.23:3055
cput client mode enabled
[10000 packets sent (localhost), 10000 packets received (192.168.10.23)]
```
Server Side
```
host2#cput -s --server-bind 192.158.10.23:3055
cput server mode enabled
[10000 packets sent (192.158.11.44), 10000 packets received (localhost)]
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
    -r, --rate-limit <rate>            set rate limit
        --server-addr <server-addr>    server / host ip::port address
        --server-bind <server-bind>    server bind ip::port address
        --source-addr <source-addr>    client source ip::port address
    -t, --thread <thread>              threads number
```
