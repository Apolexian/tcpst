use std::net::{IpAddr, Ipv4Addr};
use std::ops::Add;

use mpstrust::Role;
use pnet::packet::ip::IpNextHeaderProtocols::{self, Tcp};
use pnet::packet::{MutablePacket, Packet};

use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::tcp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use raw_socket::{Domain, Protocol, Type};
extern crate pnet;

pub struct RoleServerSystem {}
impl Role for RoleServerSystem {}

pub struct RoleServerUser {}
impl Role for RoleServerUser {}

pub struct RoleServerClient {}
impl Role for RoleServerClient {}

fn main() {
    let remote_addr = Ipv4Addr::new(127, 0, 0, 1);
    let local_addr = Ipv4Addr::new(127, 0, 0, 1);
    let remote_addr_v4 = IpAddr::V4(remote_addr);

    let _destination_port = 40000_u16;
    let source_port = 49155_u16;

    // Silly trick to make the kernel not process TCP packets
    // this is used in combination with `iptables -A OUTPUT -p tcp --tcp-flags RST RST -j DROP`,
    // which drops any outgoing RST segments that the kernel tries to send.
    // This socket is never used again after but it means that the kernel will not try to process incoming segments.
    // https://stackoverflow.com/questions/31762305/prevent-kernel-from-processing-tcp-segments-bound-to-a-raw-socket
    let socket =
        raw_socket::RawSocket::new(Domain::ipv4(), Type::stream(), Some(Protocol::tcp())).unwrap();
    socket.bind(("127.0.0.1", source_port)).unwrap();

    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Tcp));
    let (mut tx, mut rx) = match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!(
            "An error occurred when creating the transport channel: {}",
            e
        ),
    };

    let mut iter = tcp_packet_iter(&mut rx);
    let got_syn = false;
    loop {
        match iter.next() {
            Ok((packet, _)) => {
                // if packet is not for us then something went wrong
                if !(packet.get_destination() == 49155) {
                    continue;
                }
                if !got_syn {
                    eprintln!("got packet...");
                    eprintln!("{:?}", packet.get_destination());
                    eprintln!("{:?}", packet.get_sequence());
                    eprintln!("{:?}", packet.get_source());

                    let mut vec: Vec<u8> = vec![0; packet.packet().len()];
                    let mut new_packet = MutableTcpPacket::new(&mut vec[..]).unwrap();
                    new_packet.set_flags(TcpFlags::ACK | TcpFlags::SYN);
                    new_packet.set_sequence(1);
                    new_packet.set_acknowledgement(packet.get_sequence().add(1));
                    new_packet.set_source(packet.get_destination());
                    new_packet.set_destination(packet.get_source());
                    new_packet.set_window(4015);
                    new_packet.set_data_offset(8);
                    let checksum =
                        ipv4_checksum(&new_packet.to_immutable(), &local_addr, &remote_addr);
                    new_packet.set_checksum(checksum);

                    // Send the packet
                    match tx.send_to(new_packet, remote_addr_v4) {
                        Ok(n) => assert_eq!(n, packet.packet().len()),
                        Err(e) => panic!("failed to send packet: {}", e),
                    }
                } else {
                    eprintln!("todo!");
                }
            }
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}
