use std::marker::PhantomData;

use etherparse::{ReadError, TcpHeader};
use tun_tap::Iface;

use crate::{Action, Message, OfferOne, Role, SelectOne};

pub struct Ack {
    header: TcpHeader,
    payload: Vec<u8>,
}
impl Message for Ack {}
impl AckMessage for Ack{}

pub trait SynMessage: Message {}
pub trait AckMessage: Message {}

pub struct Syn {
    header: TcpHeader,
    payload: Vec<u8>,
}
impl Message for Syn {}
impl SynMessage for Syn{}

pub struct Nic<R1, R2> {
    nic: Iface,
    phantom: PhantomData<(R1, R2)>,
}

impl<R1, R2> Nic<R1, R2> {
    pub fn new(nic: Iface) -> Self {
        Nic {
            nic,
            phantom: PhantomData::default(),
        }
    }

    pub fn to_ack_message(slice: Vec<u8>) -> Result<Ack, ReadError> {
        let packet = TcpHeader::from_slice(&slice)?;
        // make sure only ACK flag is set
        if !packet.0.ack || (packet.0.rst || packet.0.syn || packet.0.fin) {
            return Err(ReadError::UnexpectedEndOfSlice(0));
        }
        Ok(Ack {
            header: packet.0,
            payload: packet.1.to_vec(),
        })
    }

    pub fn from_ack_message(ack: Ack) -> Vec<u8> {
        let mut packet: Vec<u8> = vec![];
        // Etherparse does not implement TcpHeader back to slice so we would need to
        // manually push headers and other bits here.
        // This is omitted as an implementation detail.
        packet.push(ack.header.destination_port.try_into().unwrap());
        let mut payload = ack.payload;
        packet.append(&mut payload);
        packet
    }

    pub fn to_syn_message(slice: Vec<u8>) -> Result<Syn, ReadError> {
        let packet = TcpHeader::from_slice(&slice)?;
        // make sure only SYN flag is set
        if !packet.0.syn || (packet.0.rst || packet.0.ack || packet.0.fin) {
            return Err(ReadError::UnexpectedEndOfSlice(0));
        }
        Ok(Syn {
            header: packet.0,
            payload: packet.1.to_vec(),
        })
    }

    pub fn from_syn_message(ack: Syn) -> Vec<u8> {
        let mut packet: Vec<u8> = vec![];
        // Etherparse does not implement TcpHeader back to slice so we would need to
        // manually push headers and other bits here.
        // This is omitted as an implementation detail.
        packet.push(ack.header.destination_port.try_into().unwrap());
        let mut payload = ack.payload;
        packet.append(&mut payload);
        packet
    }
}

#[must_use]
pub fn offer_one_ack<'a, R1, R2, M, A>(_o: OfferOne<R1, M, A>, channel: &Nic<R2, R1>) -> (Ack, A)
where
    M: AckMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let mut buf = vec![0; 1500];
    let read = channel.nic.recv(&mut buf).unwrap();
    let ack = Nic::<R1, R2>::to_ack_message(buf[..read].to_vec()).unwrap();
    (ack, A::new())
}

#[must_use]
pub fn offer_one_syn<'a, R1, R2, M, A>(_o: OfferOne<R1, M, A>, channel: &Nic<R2, R1>) -> (Syn, A)
where
    M: SynMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let mut buf = vec![0; 1500];
    let read = channel.nic.recv(&mut buf).unwrap();
    let ack = Nic::<R1, R2>::to_syn_message(buf[..read].to_vec()).unwrap();
    (ack, A::new())
}

#[must_use]
pub fn select_one_ack<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: Ack,
    channel: &Nic<R1, R2>,
) -> A
where
    M: AckMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = Nic::<R1, R2>::from_ack_message(message);
    channel.nic.send(&packet).unwrap();
    A::new()
}

#[must_use]
pub fn select_one_syn<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: Syn,
    channel: &Nic<R1, R2>,
) -> A
where
    M: SynMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = Nic::<R1, R2>::from_syn_message(message);
    channel.nic.send(&packet).unwrap();
    A::new()
}

#[cfg(test)]
mod nic_tests {
    use packet::{
        tcp::{self, Flags},
        Builder,
    };
    use std::thread;

    use crate::{
        nic::{offer_one_ack, offer_one_syn, select_one_ack, select_one_syn, Ack, Nic, Syn},
        Action, End, OfferOne, Role, SelectOne,
    };

    #[derive(Clone)]
    struct RoleA {}
    impl Role for RoleA {}

    #[derive(Clone)]
    struct RoleB {}
    impl Role for RoleB {}

    #[derive(Clone)]
    struct RoleC {}
    impl Role for RoleC {}

    #[test]
    fn simple_protocol() {
        type StA = OfferOne<RoleB, Ack, OfferOne<RoleB, Syn, OfferOne<RoleC, Ack, End>>>;
        type StB = SelectOne<RoleA, Ack, SelectOne<RoleA, Syn, End>>;
        type StC = SelectOne<RoleA, Ack, End>;

        let nic_b_to_a = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun).unwrap();
        let nic_a_to_b = tun_tap::Iface::without_packet_info("tun1", tun_tap::Mode::Tun).unwrap();
        let nic_c_to_a = tun_tap::Iface::without_packet_info("tun2", tun_tap::Mode::Tun).unwrap();
        let nic_a_to_c = tun_tap::Iface::without_packet_info("tun3", tun_tap::Mode::Tun).unwrap();

        let channel_b_to_a = Nic::<RoleB, RoleA>::new(nic_b_to_a);
        let channel_a_to_b = Nic::<RoleA, RoleB>::new(nic_a_to_b);
        let channel_c_to_a = Nic::<RoleA, RoleC>::new(nic_c_to_a);
        let channel_a_to_c = Nic::<RoleC, RoleA>::new(nic_a_to_c);

        let a = StA::new();
        let b = StB::new();
        let c = StC::new();

        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let (_, cont) = offer_one_ack(a, &channel_a_to_b);
                let (_, cont) = offer_one_syn(cont, &channel_a_to_b);
                let (_, cont) = offer_one_ack(cont, &channel_c_to_a);
                cont.close();
            });
            let thread_b = scope.spawn(|| {
                let ack_packet = tcp::Builder::default();
                let mut flags = Flags::empty();
                flags.insert(Flags::ACK);
                let ack_packet = ack_packet.flags(flags).unwrap();
                let ack_packet = ack_packet.build().unwrap();
                let ack_message = Nic::<RoleB, RoleA>::to_ack_message(ack_packet).unwrap();

                let syn_packet = tcp::Builder::default();
                let mut flags = Flags::empty();
                flags.insert(Flags::SYN);
                let syn_packet = syn_packet.flags(flags).unwrap();
                let syn_packet = syn_packet.build().unwrap();
                let syn_message = Nic::<RoleB, RoleA>::to_syn_message(syn_packet).unwrap();

                let cont = select_one_ack(b, ack_message, &channel_b_to_a);
                let cont = select_one_syn(cont, syn_message, &channel_b_to_a);
                cont.close();
            });
            let thread_c = scope.spawn(|| {
                let ack_packet = tcp::Builder::default();
                let mut flags = Flags::empty();
                flags.insert(Flags::ACK);
                let ack_packet = ack_packet.flags(flags).unwrap();
                let ack_packet = ack_packet.build().unwrap();
                let ack_message = Nic::<RoleB, RoleA>::to_ack_message(ack_packet).unwrap();

                let cont = select_one_ack(c, ack_message, &channel_a_to_c);
                cont.close();
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
            thread_c.join().unwrap();
        });
    }
}
