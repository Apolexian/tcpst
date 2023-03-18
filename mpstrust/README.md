## TODO

## TCP handshake demo

We test that the TCP handshake works in the following way.
The binary in this rust projects acts as the TCP server.
It uses a layer 4 interface provided by `libpnet` to read incoming TCP packets.
We filter traffic by port since we only care about our one connection.

You can use tcpdump to examine `lo` on the appropriate port using `tcpdump -i lo -v 'port 541'`.
After this you can try to connect to the TCP server using any means.
We use the netcat utility: `nc 192.168.0.2 541`.

Note that this exists purely for demonstration purposes and does not have any form of complete or correct TCP implementation.
The server works only to the point of establishing a connection and makes assumptions about reliability.
