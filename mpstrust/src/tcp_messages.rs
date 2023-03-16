use pnet::packet::tcp::MutableTcpPacket;

use crate::{pnet_channel::PNetTcpMessage, Message};

/// Examples of internal representations of TCP message types
/// We assume a well behaved parser, in reality this would need
/// more checks and would probably have a more useful internal representation.
/// As our intended purpose is to complete a handshake without worrying about
/// retransmission, this suffices.

pub struct Ack<'a> {
    ack: u32,
    packet: MutableTcpPacket<'a>,
}

impl<'a> Message for Ack<'a> {}

impl<'a> Ack<'a> {
    pub fn new(packet: MutableTcpPacket<'a>) -> Self {
        Ack {
            ack: packet.get_acknowledgement(),
            packet,
        }
    }
}

impl<'a> PNetTcpMessage<'a> for Ack<'a> {
    fn from_pnet_tcp_packet(packet: MutableTcpPacket<'a>) -> Self {
        Ack::new(packet)
    }

    fn to_pnet_tcp_packet(self) -> MutableTcpPacket<'a> {
        self.packet
    }
}

pub struct Syn<'a> {
    packet: MutableTcpPacket<'a>,
}

impl Message for Syn<'_> {}

impl<'a> PNetTcpMessage<'a> for Syn<'a> {
    fn from_pnet_tcp_packet(packet: MutableTcpPacket<'a>) -> Self {
        Syn { packet }
    }

    fn to_pnet_tcp_packet(self) -> MutableTcpPacket<'a> {
        self.packet
    }
}

pub struct SynAck<'a> {
    packet: MutableTcpPacket<'a>,
}

impl Message for SynAck<'_> {}

impl<'a> PNetTcpMessage<'a> for SynAck<'a> {
    fn from_pnet_tcp_packet(packet: MutableTcpPacket<'a>) -> Self {
        SynAck { packet }
    }

    fn to_pnet_tcp_packet(self) -> MutableTcpPacket<'a> {
        self.packet
    }
}
