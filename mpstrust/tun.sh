#!/bin/bash
ext=$?
if [[ $ext -ne 0 ]]; then
    exit $ext
fi

sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

sudo ip addr add 192.168.0.2/24 dev tun1
sudo ip link set up dev tun1