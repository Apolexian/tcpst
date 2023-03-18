use mpstrust::{Role};
use pnet::packet::ip::IpNextHeaderProtocols;

use pnet::packet::{Packet};
use pnet::transport::tcp_packet_iter;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use pnet::transport::{transport_channel};

pub struct RoleServerSystem {}
impl Role for RoleServerSystem {}

pub struct RoleServerUser {}
impl Role for RoleServerUser {}

pub struct RoleServerClient {}
impl Role for RoleServerClient {}

fn main() {
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Tcp));
    let (_, mut rx) = match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!(
            "An error occurred when creating the transport channel: {}",
            e
        ),
    };

    let mut iter = tcp_packet_iter(&mut rx);
    loop {
        match iter.next() {
            Ok((packet, _addr)) => {
                let parsed = smoltcp::wire::TcpPacket::new_checked(packet.packet()).unwrap();
                // filter out other broadcast traffic
                if parsed.dst_port() != 541 {
                    continue;
                }
                eprintln!("{:?}", parsed.syn());
            }
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}
