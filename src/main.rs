use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use proto::{EtherType, ProtocolType};
use std::io;
use tun_tap::Iface;

const FRAME_ETHER_TYPE_START: usize = 2;
const FRAME_ETHER_TYPE_END: usize = 3;
const FRAME_RAW_PROTO_START: usize = 4;

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        // based on section 3.2 Frame format
        // https://www.kernel.org/doc/Documentation/networking/tuntap.txt
        let read = nic.recv(&mut buf[..])?;
        let ether_type: EtherType =
            u16::from_be_bytes([buf[FRAME_ETHER_TYPE_START], buf[FRAME_ETHER_TYPE_END]]).into();

        // for now just ignore anything that is not IPv4
        if ether_type != EtherType::IPv4 {
            continue;
        }

        let ipv4_header_slice = Ipv4HeaderSlice::from_slice(&buf[FRAME_RAW_PROTO_START..read])
            .expect("Failed to parse IPv4 header slice");
        let protocol: ProtocolType = ipv4_header_slice.protocol().into();

        // ignore anything that is not a TCP packet
        if protocol != ProtocolType::TCP {
            let tcp_header_slice =
                TcpHeaderSlice::from_slice(&buf[4 + ipv4_header_slice.slice().len()..])
                    .expect("Failed to parse TCP header slice");
            eprintln!("{:?}", tcp_header_slice.options());
        }
    }
}

mod proto;
