# TCP-ST

Overleaf link: https://www.overleaf.com/6549774965nbxrbvyffctk 

## Model

The MPST model of TCP is provided in the `tcp.ctx` file.
The model needs the TCP payload fork of the [mpstk tool](https://github.com/Apolexian/mpstk) to execute.
A compiled binary is provided in the `bin` directory.
Note that you will still need to install `mclr2` and clone the `mpstk` fork to run this.
To run the model use the `run_model.sh` script.

## TCP handshake demo

The `mpstrust` directory contains the MPST implementation and example TCP server.
The source includes an example of an implementation of a very basic TCP server that completes the opening handshake and issues connection close.
It uses a layer 4 interface provided by `libpnet` to read incoming TCP packets.
Netcat was used to connect to the server and tcpdump to observe packets.
The server only reads packets on port 49155.

To run the example server you can use the provided script:

```
chmod +x run.sh
sudo ./run.sh
```

Example output from tcpdump:

```
12:38:28.734403 IP (tos 0x0, ttl 64, id 40379, offset 0, flags [DF], proto TCP (6), length 60)
    localhost.52400 > localhost.49155: Flags [S], cksum 0xfe30 (incorrect -> 0x5700), seq 3708127670, win 65495, options [mss 65495,sackOK,TS val 2302245975 ecr 0,nop,wscale 7], length 0
12:38:28.734654 IP (tos 0x0, ttl 64, id 56685, offset 0, flags [DF], proto TCP (6), length 60)
    localhost.49155 > localhost.52400: Flags [S.], cksum 0x6e71 (correct), seq 1, ack 3708127671, win 65495, options [eol], length 0
12:38:28.734821 IP (tos 0x0, ttl 64, id 40380, offset 0, flags [DF], proto TCP (6), length 40)
    localhost.52400 > localhost.49155: Flags [.], cksum 0xfe1c (incorrect -> 0xbe86), ack 1, win 65495, length 0
12:38:28.735183 IP (tos 0x0, ttl 64, id 56686, offset 0, flags [DF], proto TCP (6), length 40)
    localhost.49155 > localhost.52400: Flags [F.], cksum 0xbe84 (correct), seq 1, ack 2, win 65495, length 0
```

Example output of netcat:

```
Connection to 127.0.0.1 49155 port [tcp/*] succeeded!
```

Note that netcat's closing behaviour is to stay in a half-closed connection until it is terminated, so it will not send the final packets to close the connection.
