use std::net::{IpAddr, Ipv4Addr};

use mpstrust::Role;
use pnet::packet::ip::IpNextHeaderProtocols::{self, Tcp};
use pnet::packet::{MutablePacket, Packet};

use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::tcp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
extern crate pnet;

pub struct RoleServerSystem {}
impl Role for RoleServerSystem {}

pub struct RoleServerUser {}
impl Role for RoleServerUser {}

pub struct RoleServerClient {}
impl Role for RoleServerClient {}

fn main() {
    let addr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1));
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Tcp));
    let (mut tx, mut rx) = match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!(
            "An error occurred when creating the transport channel: {}",
            e
        ),
    };

    let mut iter = tcp_packet_iter(&mut rx);
    let mut buffer = [0; 4095];
    let mut syn_packet = MutableTcpPacket::new(&mut buffer).unwrap();
    syn_packet.set_flags(TcpFlags::SYN);
    syn_packet.set_sequence(1);
    //  iptables -t raw -A PREROUTING -p tcp --dport 41955 -j DROP 
    syn_packet.set_source(49155);
    syn_packet.set_destination(541);
    syn_packet.set_window(4015);
    syn_packet.set_data_offset(8);
    let checksum = ipv4_checksum(
        &syn_packet.to_immutable(),
        &Ipv4Addr::new(192, 168, 0, 1),
        &Ipv4Addr::new(192, 168, 0, 1),
    );
    syn_packet.set_checksum(checksum);
    tx.send_to(syn_packet, addr).unwrap();

    match iter.next() {
        Ok((packet, _)) => {
            // if packet is not for us then something went wrong
            if !packet.get_destination() == 49155 {
                panic!("Deal with packet from wrong source...")
            }
            if !packet.get_acknowledgement() == 2 {
                panic!("Deal with wrong ack numbers here...")
            }

            let mut vec: Vec<u8> = vec![0; packet.packet().len()];
            let mut new_packet = MutableTcpPacket::new(&mut vec[..]).unwrap();
            new_packet.set_flags(TcpFlags::ACK);
            new_packet.set_sequence(packet.get_acknowledgement() + 1);
            new_packet.set_source(packet.get_destination());
            new_packet.set_destination(packet.get_source());
            new_packet.set_window(4015);
            new_packet.set_data_offset(8);
            let checksum = ipv4_checksum(
                &new_packet.to_immutable(),
                &Ipv4Addr::new(192, 168, 0, 1),
                &Ipv4Addr::new(192, 168, 0, 1),
            );
            new_packet.set_checksum(checksum);

            // Send the packet
            match tx.send_to(new_packet, addr) {
                Ok(n) => assert_eq!(n, packet.packet().len()),
                Err(e) => panic!("failed to send packet: {}", e),
            }
        }
        Err(e) => {
            // If an error occurs, we can handle it here
            panic!("An error occurred while reading: {}", e);
        }
    }
}
