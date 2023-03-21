#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::net::{IpAddr, Ipv4Addr};
use std::ops::Add;

use mpstrust::Role;
use pnet::packet::ip::IpNextHeaderProtocols::{self};
use pnet::packet::{Packet};

use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags};
use pnet::transport::tcp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use raw_socket::{Domain, Protocol, Type};
extern crate pnet;

pub struct RoleServerSystem;
impl Role for RoleServerSystem {}

pub struct RoleServerUser;
impl Role for RoleServerUser {}

pub struct RoleServerClient;
impl Role for RoleServerClient {}

fn main() {
    let remote_addr = Ipv4Addr::new(127, 0, 0, 1);
    let local_addr = Ipv4Addr::new(127, 0, 0, 1);
    let remote_addr_v4 = IpAddr::V4(remote_addr);

    let _destination_port = 40000u16;
    let source_port = 49155u16;

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
    loop {
        match iter.next() {
            Ok((packet, _)) => {
                // ignore packets that are not for us
                if packet.get_destination() != 49155 {
                    continue;
                }

                // got a SYN, start handshake
                if packet.get_flags() == TcpFlags::SYN {
                    // construct SYN-ACK packet
                    let mut vec: Vec<u8> = vec![0; packet.packet().len()];
                    let mut new_packet = MutableTcpPacket::new(&mut vec).unwrap();
                    new_packet.set_flags(TcpFlags::ACK | TcpFlags::SYN);
                    new_packet.set_sequence(1);
                    new_packet.set_acknowledgement(packet.get_sequence().add(1));
                    new_packet.set_source(packet.get_destination());
                    new_packet.set_destination(packet.get_source());
                    new_packet.set_window(packet.get_window());
                    new_packet.set_data_offset(packet.get_data_offset());
                    let checksum =
                        ipv4_checksum(&new_packet.to_immutable(), &local_addr, &remote_addr);
                    new_packet.set_checksum(checksum);

                    // Send the packet
                    match tx.send_to(new_packet, remote_addr_v4) {
                        Ok(n) => assert_eq!(n, packet.packet().len()),
                        Err(e) => panic!("failed to send packet: {e}"),
                    }
                } else {
                    // whenever we get any other packet we will just FIN-ACK
                    let mut vec: Vec<u8> = vec![0; packet.packet().len()];
                    let mut new_packet = MutableTcpPacket::new(&mut vec).unwrap();
                    new_packet.set_flags(TcpFlags::ACK | TcpFlags::FIN);
                    new_packet.set_sequence(packet.get_acknowledgement());
                    new_packet.set_acknowledgement(packet.get_sequence().add(1));
                    new_packet.set_source(packet.get_destination());
                    new_packet.set_destination(packet.get_source());
                    new_packet.set_window(packet.get_window());
                    new_packet.set_data_offset(packet.get_data_offset());
                    let checksum =
                        ipv4_checksum(&new_packet.to_immutable(), &local_addr, &remote_addr);
                    new_packet.set_checksum(checksum);
                    // Send the packet
                    match tx.send_to(new_packet, remote_addr_v4) {
                        Ok(n) => assert_eq!(n, packet.packet().len()),
                        Err(e) => panic!("failed to send packet: {e}"),
                    }
                    // Get an ACK for our FIN-ACK and ACK it
                    // Note that the default behaviour of netcat is to stay in a
                    // half-open connection until you close netcat's stdin (Ctrl-D).
                    // Once netcat's stdin is closed it will issue the final FIN.
                    // We do not handle this here and leave the example content
                    // with just issuing its FIN-ACK.
                    match iter.next() {
                        Ok((packet, _)) => {
                            if packet.get_flags() & TcpFlags::ACK == TcpFlags::ACK {
                                let mut vec: Vec<u8> = vec![0; packet.packet().len()];
                                let mut new_packet = MutableTcpPacket::new(&mut vec).unwrap();
                                new_packet.set_flags(TcpFlags::ACK);
                                new_packet.set_sequence(packet.get_acknowledgement());
                                new_packet.set_acknowledgement(packet.get_sequence().add(1));
                                new_packet.set_source(packet.get_destination());
                                new_packet.set_destination(packet.get_source());
                                new_packet.set_window(packet.get_window());
                                new_packet.set_data_offset(packet.get_data_offset());
                                let checksum = ipv4_checksum(
                                    &new_packet.to_immutable(),
                                    &local_addr,
                                    &remote_addr,
                                );
                                new_packet.set_checksum(checksum);
                                
                                match tx.send_to(new_packet, remote_addr_v4) {
                                    Ok(_) => {
                                        println!("Connection closed successfully");
                                        return;
                                    }
                                    Err(e) => panic!("failed to send packet: {e}"),
                                }
                            }
                        }
                        Err(e) => {
                            panic!("An error occurred while reading: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                panic!("An error occurred while reading: {e}");
            }
        }
    }
}
