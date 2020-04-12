# cput
circuit throughput 

Client Side
```
cput -h 127.0.0.1:3077 -c 100000 -t 10 -r 1000
```
Server Side
```
cput -s -b 0.0.0.0:3077
```

```
cput --help
circuit throughput check 0.0.1

USAGE:
    cput [FLAGS] [OPTIONS]

FLAGS:
        --help       Prints help information
    -s, --server     enable server mode
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <bind>        bind address and port
    -c, --count <count>      count numbers
    -h, --host <host>        host address and port
    -r, --rate <rate>        rate of packets per second
    -t, --thread <thread>    threads number
```
