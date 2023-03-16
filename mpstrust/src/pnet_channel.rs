use std::net::IpAddr;

use pnet::{
    packet::{tcp::MutableTcpPacket, Packet},
    transport::{tcp_packet_iter, TransportReceiver, TransportSender},
};

use crate::Message;

pub trait PNetTcpMessage<'a>: Message {
    fn from_pnet_tcp_packet(packet: MutableTcpPacket<'a>) -> Self;
    fn to_pnet_tcp_packet(self) -> MutableTcpPacket<'a>;
}

pub struct TransportChannel {
    send: TransportSender,
    recv: TransportReceiver,
    destination: IpAddr,
}

impl TransportChannel {
    pub fn send(&mut self, packet: MutableTcpPacket) -> usize {
        self.send.send_to(packet, self.destination).unwrap()
    }

    pub fn recv(&mut self) -> MutableTcpPacket {
        let mut iter = tcp_packet_iter(&mut self.recv);
        loop {
            match iter.next() {
                Ok((packet, _)) => {
                    // Allocate enough space for a new packet
                    let vec: Vec<u8> = vec![0; packet.packet().len()];
                    let new_packet = MutableTcpPacket::owned(vec[..].to_vec()).unwrap();
                    return new_packet;
                }
                Err(e) => {
                    // If an error occurs, we can handle it here
                    panic!("An error occurred while reading: {}", e);
                }
            }
        }
    }
}
