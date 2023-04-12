/**
 * Copyright 2023, Ivan Nikitin.
 * This file is part of TCP-ST.
 *
 * TCP-ST is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
 *
 * TCP-ST is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with TCP-ST.
 * If not, see <https://www.gnu.org/licenses/>.
 *
 */
use pnet::{
    packet::{tcp::MutableTcpPacket, Packet},
    transport::{TcpTransportChannelIterator, TransportSender},
};

use crate::{Branch, Message, Role, SessionTypedChannel};
use std::{marker::PhantomData, net::Ipv4Addr};

/// [NetChannel] is a session-typed communication channel that uses
/// libpnet [TransportSender] and [TcpTransportChannelIterator] under the hood.
/// [NetChannel] behaves as any other session-typed channels and implements [SessionTypedChannel].
pub struct NetChannel<'a, R1, R2>
where
    R1: Role,
    R2: Role,
{
    rx: TcpTransportChannelIterator<'a>,
    tx: TransportSender,
    remote_addr: Ipv4Addr,
    phantom: PhantomData<(R1, R2)>,
}

impl<R1, R2> SessionTypedChannel<R1, R2> for NetChannel<'_, R1, R2>
where
    R1: Role,
    R2: Role,
{
    #[must_use]
    fn offer_one<M, A>(&mut self, _o: crate::OfferOne<R2, M, A>) -> (M, A)
    where
        M: crate::Message + 'static,
        A: crate::Action + 'static,
        R1: Role,
        R2: Role,
    {
        loop {
            match self.rx.next() {
                Ok((packet, _)) => {
                    // ignore packets that are not for us
                    if packet.get_destination() != 49155 {
                        continue;
                    }
                    let slice = packet.packet().to_vec();
                    let message = M::from_net_representation(slice);
                    return (message, A::new());
                }
                Err(e) => {
                    panic!("An error occurred while reading: {e}");
                }
            }
        }
    }

    fn select_one<M, A>(&mut self, _o: crate::SelectOne<R2, M, A>, message: M) -> A
    where
        M: crate::Message,
        A: crate::Action,
        R1: Role,
        R2: Role,
    {
        let mut packet = message.to_net_representation();
        let packet_inner = MutableTcpPacket::new(&mut packet[..]).unwrap();
        match self
            .tx
            .send_to(packet_inner, std::net::IpAddr::V4(self.remote_addr))
        {
            Ok(_) => {
                return A::new();
            }
            Err(e) => panic!("failed to send packet: {e}"),
        }
    }

    fn offer_two<M1, M2, A1, A2>(
        &mut self,
        _o: crate::OfferTwo<R2, M1, M2, A1, A2>,
        picker: Box<dyn Fn() -> bool>,
    ) -> crate::Branch<(M1, A1), (M2, A2)>
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        let choice = picker();
        match choice {
            true => {
                loop {
                    match self.rx.next() {
                        Ok((packet, _)) => {
                            // ignore packets that are not for us
                            if packet.get_destination() != 49155 {
                                continue;
                            }
                            let slice = packet.packet().to_vec();
                            let message = M1::from_net_representation(slice);
                            return Branch::Left((message, A1::new()));
                        }
                        Err(e) => {
                            panic!("An error occurred while reading: {e}");
                        }
                    }
                }
            }
            false => loop {
                match self.rx.next() {
                    Ok((packet, _)) => {
                        if packet.get_destination() != 49155 {
                            continue;
                        }
                        let slice = packet.packet().to_vec();
                        let message = M2::from_net_representation(slice);
                        return Branch::Right((message, A2::new()));
                    }
                    Err(e) => {
                        panic!("An error occurred while reading: {e}");
                    }
                }
            },
        }
    }

    fn select_left<M1, M2, A1, A2>(
        &mut self,
        _o: crate::SelectTwo<R2, M1, M2, A1, A2>,
        message: M1,
    ) -> A1
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        let mut packet = message.to_net_representation();
        let packet_inner = MutableTcpPacket::new(&mut packet[..]).unwrap();
        match self
            .tx
            .send_to(packet_inner, std::net::IpAddr::V4(self.remote_addr))
        {
            Ok(_) => {
                return A1::new();
            }
            Err(e) => panic!("failed to send packet: {e}"),
        }
    }

    fn select_right<M1, M2, A1, A2>(
        &mut self,
        _o: crate::SelectTwo<R2, M1, M2, A1, A2>,
        message: M2,
    ) -> A2
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        let mut packet = message.to_net_representation();
        let packet_inner = MutableTcpPacket::new(&mut packet[..]).unwrap();
        match self
            .tx
            .send_to(packet_inner, std::net::IpAddr::V4(self.remote_addr))
        {
            Ok(_) => {
                return A2::new();
            }
            Err(e) => panic!("failed to send packet: {e}"),
        }
    }

    fn close(self, _end: crate::End) {
        drop(self);
    }
}

impl<'a, R1, R2> NetChannel<'a, R1, R2>
where
    R1: Role,
    R2: Role,
{
    pub fn new(
        rx: TcpTransportChannelIterator<'a>,
        tx: TransportSender,
        remote_addr: Ipv4Addr,
    ) -> Self {
        NetChannel {
            rx,
            tx,
            remote_addr,
            phantom: PhantomData::default(),
        }
    }
}

/// [Syn] is the specific message type for a packet with
/// the SYN flag set. We assume a well-behaved parser and
/// leave the parsing implementation to the user. Hence,
/// this demonstration has an extremely simplistic layout of
/// message structs and does **no** error handling.
/// Hence, it is possible to construct a [Syn] message out of
/// a wrong packet. This should ideally be handled by checking
/// that correct flags are set and returning errors.
pub struct Syn {
    pub packet: Vec<u8>,
}

impl Message for Syn {
    fn to_net_representation(self) -> Vec<u8> {
        self.packet
    }

    fn from_net_representation(packet: Vec<u8>) -> Self {
        Syn { packet }
    }
}

/// [SynAck] is the specific message type for a packet with
/// the SYN flag set. We assume a well-behaved parser and
/// leave the parsing implementation to the user. Hence,
/// this demonstration has an extremely simplistic layout of
/// message structs and does **no** error handling.
/// Hence, it is possible to construct a [SynAck] message out of
/// a wrong packet. This should ideally be handled by checking
/// that correct flags are set and returning errors.
pub struct SynAck {
    pub packet: Vec<u8>,
}

impl Message for SynAck {
    fn to_net_representation(self) -> Vec<u8> {
        self.packet
    }

    fn from_net_representation(packet: Vec<u8>) -> Self {
        SynAck { packet }
    }
}

/// [Ack] is the specific message type for a packet with
/// the SYN flag set. We assume a well-behaved parser and
/// leave the parsing implementation to the user. Hence,
/// this demonstration has an extremely simplistic layout of
/// message structs and does **no** error handling.
/// Hence, it is possible to construct a [Ack] message out of
/// a wrong packet. This should ideally be handled by checking
/// that correct flags are set and returning errors.
pub struct Ack {
    pub packet: Vec<u8>,
}

impl Message for Ack {
    fn to_net_representation(self) -> Vec<u8> {
        self.packet
    }

    fn from_net_representation(packet: Vec<u8>) -> Self {
        Ack { packet }
    }
}

/// [FinAck] is the specific message type for a packet with
/// the SYN flag set. We assume a well-behaved parser and
/// leave the parsing implementation to the user. Hence,
/// this demonstration has an extremely simplistic layout of
/// message structs and does **no** error handling.
/// Hence, it is possible to construct a [FinAck] message out of
/// a wrong packet. This should ideally be handled by checking
/// that correct flags are set and returning errors.
pub struct FinAck {
    pub packet: Vec<u8>,
}

impl Message for FinAck {
    fn to_net_representation(self) -> Vec<u8> {
        self.packet
    }

    fn from_net_representation(packet: Vec<u8>) -> Self {
        FinAck { packet }
    }
}
