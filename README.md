# WIP

# Useful commands

Running the protocol via tun/tap:

```shell
chmod +x run.sh
./run.sh
```

Ping some IP in the created tun range:

```shell
ping -I tun0 192.168.0.2
```

Start TCP connection:

```shell
nc 192.168.0.2
```