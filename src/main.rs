use proto::EtherType;
use std::io;
use tun_tap::Iface;

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        // based on section 3.2 Frame format
        // https://www.kernel.org/doc/Documentation/networking/tuntap.txt
        let read = nic.recv(&mut buf[..])?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto: EtherType = u16::from_be_bytes([buf[2], buf[3]]).into();
        eprintln!("proto: {:?}", proto);
        if proto != EtherType::IPv4 {
            continue;
        }
        eprintln!(
            "Nic got proto {:?} and flags {:?} with raw protocol {:?}",
            proto,
            flags,
            &buf[4..read]
        );
    }
}

mod proto;
