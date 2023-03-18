use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use mpstrust::{net_channel::NetChannel, Role};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::{MutablePacket, Packet};
use pnet::transport::tcp_packet_iter;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use pnet::transport::{transport_channel, udp_packet_iter};
use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::RawSocket;
use smoltcp::phy::{Device, RxToken};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetFrame, PrettyPrinter};
use std::os::unix::io::AsRawFd;
use std::{env, io};
use tun_tap::Iface;

const FRAME_ETHER_TYPE_START: usize = 2;
const FRAME_ETHER_TYPE_END: usize = 3;
const FRAME_RAW_PROTO_START: usize = 4;

/// Field in an Ethernet Frame used to determine which protocol is in the payload
/// https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml#ieee-802-numbers-1
/// We really only care about IP packets
#[derive(PartialEq)]
pub(crate) enum EtherType {
    IPv4 = 0x0800,
    IPv6 = 0x86DD,
    Undefined,
}

impl Into<EtherType> for u16 {
    fn into(self) -> EtherType {
        match self {
            0x0800 => EtherType::IPv4,
            0x86DD => EtherType::IPv6,
            _ => EtherType::Undefined,
        }
    }
}

/// https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
/// These are just some of the Protocols that we may care about and is not meant to be exhaustive
#[derive(PartialEq, Debug)]
pub(crate) enum ProtocolType {
    ICMP = 0x001,
    TCP = 0x006,
    UDP = 0x011,
    Undefined,
}

impl Into<ProtocolType> for u8 {
    fn into(self) -> ProtocolType {
        match self {
            0x001 => ProtocolType::ICMP,
            0x006 => ProtocolType::TCP,
            0x011 => ProtocolType::UDP,
            _ => ProtocolType::Undefined,
        }
    }
}

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
            Ok((packet, addr)) => {
                let parsed = smoltcp::wire::TcpPacket::new_checked(packet.packet()).unwrap();
                // filter out other broadcast traffic
                if !(parsed.dst_port() == 541) {
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
