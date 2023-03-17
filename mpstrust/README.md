## TODO

## TCP handshake demo

We test that the TCP handshake works in the following way.
The binary in this rust projects acts as the TCP server.
It uses a tun interface to listen for packets, this can be ran using the `run.sh` script.
Once that is running you can use tshark to examine `tun0` using `tshark -i tun0`.
After this you can try to connect to the TCP server using any means.
We use the netcat utility: `nc 192.168.0.2 443`.

Note that this exists purely for demonstration purposes and does not have any form of complete or correct TCP implementation.
The server works only on IPv4 and its purpose is to establish the handshake using the session typed interface.
