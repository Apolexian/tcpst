use std::{collections::VecDeque, marker::PhantomData};

use smoltcp::{
    phy::ChecksumCapabilities,
    wire::{IpAddress, Ipv4Address, TcpControl, TcpPacket, TcpRepr, TcpSeqNumber},
};

use crate::Message;

pub trait SmolMessage: Message {
    fn from_packet(buf: TcpPacket<Vec<u8>>) -> Self;
    fn packet(&self) -> &TcpPacket<Vec<u8>>;
}

// macro from https://github.com/sammko/tcpst2/blob/master/src/smol_channel.rs#L158 to generate smol message
macro_rules! check_flag {
    ($p:ident, +, $flag:ident) => {
        assert!($p.$flag(), "flag {} not set", stringify!($flag));
    };
    ($p:ident, -, $flag:ident) => {
        assert!(!$p.$flag(), "flag {} set", stringify!($flag));
    };
}

macro_rules! smol_message {
    ($name:ident $({$($tag:tt $flag:ident)* $(,)?})*) => {
        pub struct $name {
            packet: TcpPacket<Vec<u8>>,
        }
        impl Message for $name {}
        impl SmolMessage for $name {
            fn packet(&self) -> &TcpPacket<Vec<u8>> {
                &self.packet
            }

            fn from_packet(packet: TcpPacket<Vec<u8>>) -> Self {
                $($(check_flag!(packet, $tag, $flag);)*)*
                $name { packet }
            }
        }
        impl From<TcpPacket<Vec<u8>>> for $name {
            fn from(packet: TcpPacket<Vec<u8>>) -> Self {
                Self::from_packet(packet)
            }
        }
    };
}

smol_message!(Syn { +syn -ack -fin -rst });
smol_message!(SynAck { +syn +ack -fin -rst });
smol_message!(Ack { -syn +ack -fin -rst });
smol_message!(FinAck { -syn +ack +fin -rst });
smol_message!(Rst { -syn -ack -fin +rst });

#[derive(Clone, Debug)]
pub struct LocalAddr {
    pub addr: Ipv4Address,
    pub checksum_capabilities: ChecksumCapabilities,
    pub port: u16,
}

#[derive(Clone, Debug)]
pub struct RemoteAddr {
    pub addr: Ipv4Address,
    pub port: u16,
}

pub trait TcpState: Clone {}

#[macro_export]
macro_rules! MakeTcpState {
    (pub $name:ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name;
        impl TcpState for $name {}
    };
}

MakeTcpState!(pub Closed);
MakeTcpState!(pub SynSent);
MakeTcpState!(pub SynReceived);
MakeTcpState!(pub Established);
MakeTcpState!(pub FinWait1);
MakeTcpState!(pub FinWait2);
MakeTcpState!(pub CloseWait);
MakeTcpState!(pub Closing);
MakeTcpState!(pub LastAck);
MakeTcpState!(pub TimeWait);

pub struct Listen {
    pub local: LocalAddr,
}

// Keep track of key connection state variables according to https://datatracker.ietf.org/doc/html/rfc9293#section-3.3.1
#[derive(Clone, Debug)]
pub struct Tcb {
    pub snd_una: TcpSeqNumber,
    pub snd_nxt: TcpSeqNumber,
    pub snd_wnd: u16,
    pub snd_up: u16,
    pub snd_wl1: TcpSeqNumber,
    pub snd_wl2: TcpSeqNumber,
    pub iss: TcpSeqNumber,
    pub rcv_nxt: TcpSeqNumber,
    pub rcv_wnd: u16,
    pub rcv_up: u16,
    pub irs: TcpSeqNumber,
}

// The TCP struct is the overal wrapper to keep track of state, the TCB, and uses a queue to keep track of packet retransmissions.
// We use the smoltcp library to encode TCP packets.
pub struct Tcp<TcpState> {
    pub local: LocalAddr,
    pub remote: RemoteAddr,
    pub tcb: Tcb,
    pub retransmission_queue: VecDeque<TcpPacket<Vec<u8>>>,
    _phantom: PhantomData<TcpState>,
}

// https://datatracker.ietf.org/doc/html/rfc6528#section-3
// This function generates an initial random sequence number for the TCP connection.
pub fn generate_iss() -> TcpSeqNumber {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    TcpSeqNumber(rng.gen_range(0..u32::MAX).try_into().unwrap())
}

// The TCP closed state can be either created or can progress to the listen state if a passive open is requested.
impl Closed {
    pub fn new() -> Self {
        Closed
    }

    pub fn passive_open(self, local_addr: LocalAddr) -> Listen {
        Listen { local: local_addr }
    }
}

impl Listen {
    pub fn recv_syn(self, remote: Ipv4Address, syn: &Syn) -> (Tcp<SynReceived>, SynAck) {
        // Parse recieved packet using smoltcp
        let syn = TcpRepr::parse(
            &TcpPacket::new_unchecked(syn.packet().as_ref()),
            &IpAddress::from(remote),
            &IpAddress::from(self.local.addr),
            &self.local.checksum_capabilities,
        )
        .unwrap();

        // Get iss
        const ISS: TcpSeqNumber = generate_iss();

        // Create a new TCB
        let tcb: Tcb = Tcb {
            snd_una: ISS,
            snd_nxt: ISS + 1,
            snd_wnd: 0,
            snd_up: 0,
            snd_wl1: syn.seq_number,
            snd_wl2: ISS,
            iss: ISS,
            rcv_nxt: syn.seq_number + syn.segment_len(),
            rcv_wnd: 0,
            rcv_up: 0,
            irs: syn.seq_number,
        };

        // Create response packet using TcpRepr
        let mut syn_ack = TcpRepr {
            src_port: self.local.port,
            dst_port: syn.src_port,
            control: TcpControl::Syn,
            seq_number: ISS,
            ack_number: Some(tcb.rcv_nxt),
            window_len: tcb.rcv_wnd,
            payload: &[],
            window_scale: None,
            // The default TCP Maximum Segment Size is for IPv4 is 536. For IPv6 it is 1220.
            max_seg_size: Some(536),
            sack_permitted: false,
            sack_ranges: [None, None, None],
        };
        tcb.snd_nxt += syn_ack.segment_len();

        let mut packet = vec![0; syn_ack.buffer_len()];
        syn_ack.emit(
            &mut TcpPacket::new_unchecked(&mut packet),
            &IpAddress::from(remote),
            &IpAddress::from(self.local.addr),
            &self.local.checksum_capabilities,
        );
        (
            Tcp {
                local: self.local,
                remote: RemoteAddr {
                    addr: remote,
                    port: syn.src_port,
                },
                tcb,
                retransmission_queue: VecDeque::new(),
                _phantom: PhantomData,
            },
            SynAck::from_packet(TcpPacket::new_unchecked(packet)),
        )
    }
}
