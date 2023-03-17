use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use mpstrust::{net_channel::NetChannel, Role};
use std::io;
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

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
    // let st_network_channel = NetChannel::<RoleServerSystem, RoleServerClient>::new(nic);

    let mut buf = [0u8; 1504];
    loop {
        // based on section 3.2 Frame format
        // https://www.kernel.org/doc/Documentation/networking/tuntap.txt
        let read = nic.recv(&mut buf[..])?;
        let ether_type: EtherType =
            u16::from_be_bytes([buf[FRAME_ETHER_TYPE_START], buf[FRAME_ETHER_TYPE_END]]).into();

        // Ignore anything that is not IPv4
        if ether_type != EtherType::IPv4 {
            continue;
        }

        let ipv4_header_slice = Ipv4HeaderSlice::from_slice(&buf[FRAME_RAW_PROTO_START..read])
            .expect("Failed to parse IPv4 header slice");
        let protocol: ProtocolType = ipv4_header_slice.protocol().into();

        // ignore anything that is not a TCP packet
        if protocol != ProtocolType::TCP {
            continue;
        }

        let tcp_header_slice =
            TcpHeaderSlice::from_slice(&buf[4 + ipv4_header_slice.slice().len()..])
                .expect("Failed to parse TCP header slice");
        eprintln!("{:?}", tcp_header_slice.options());
    }
}
