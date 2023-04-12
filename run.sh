#!/bin/bash

OUTDIR=out
TCPDUMPFILE=out/mpstrust.pcap
NETCATFILE=out/netcat
SERVERFILE=out/server
MANIFEST=mpstrust/Cargo.toml
BINARY=mpstrust/target/release/mpstrust
PORT=49155
IP=127.0.0.1

mkdir -p $OUTDIR
touch $TCPDUMPFILE
touch $NETCATFILE
touch $SERVERFILE

sudo iptables -A OUTPUT -p tcp --tcp-flags RST RST -j DROP

RUSTFLAGS="-C target-cpu=native" cargo build --release --manifest-path=$MANIFEST
chmod +x $BINARY


sudo tcpdump -c4 -i lo -v 'port' $PORT -w $TCPDUMPFILE &
sleep 5

sudo ./$BINARY &
sudo -i nc $IP -v $PORT -w 1 & > $NETCATFILE


sleep 2;
tcpdump -r $TCPDUMPFILE 
sudo iptables -D OUTPUT -p tcp --tcp-flags RST RST -j DROP
exit 0
