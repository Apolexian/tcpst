use std::marker::PhantomData;

use pnet_channel::{TransportChannel, PNetTcpMessage};

// Supporting traits

pub trait Action: Send {
    fn new() -> Self
    where
        Self: Sized;
}

pub trait Role {}

pub trait Message: Send {}

// Session action types

pub struct OfferOne<R, M, A>
where
    M: Message,
    A: Action,
    R: Role,
{
    phantom: PhantomData<(R, M, A)>,
}

impl<R, M, A> Action for OfferOne<R, M, A>
where
    M: Message,
    A: Action,
    R: Role + std::marker::Send,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        OfferOne {
            phantom: PhantomData,
        }
    }
}

pub struct SelectOne<R, M, A>
where
    M: Message,
    A: Action,
    R: Role,
{
    phantom: PhantomData<(R, M, A)>,
}

impl<R, M, A> Action for SelectOne<R, M, A>
where
    M: Message,
    A: Action,
    R: Role + std::marker::Send,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        SelectOne {
            phantom: PhantomData,
        }
    }
}

pub struct OfferTwo<R, M1, M2, A1, A2>
where
    R: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(R, M1, M2, A1, A2)>,
}

impl<R, M1, M2, A1, A2> Action for OfferTwo<R, M1, M2, A1, A2>
where
    R: Role + std::marker::Send,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        OfferTwo {
            phantom: PhantomData::default(),
        }
    }
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

pub struct SelectTwo<R, M1, M2, A1, A2>
where
    R: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(R, M1, M2, A1, A2)>,
}

impl<R, M1, M2, A1, A2> Action for SelectTwo<R, M1, M2, A1, A2>
where
    R: Role + std::marker::Send,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        SelectTwo {
            phantom: PhantomData::default(),
        }
    }
}

pub struct End {}

impl Action for End {
    fn new() -> Self
    where
        Self: Sized,
    {
        End {}
    }
}

impl End {
    pub fn close<R1, R2>(&self, channel: TransportChannel)
    where
        R1: Role,
        R2: Role,
    {
        drop(channel);
        drop(self);
    }
}

#[must_use]
pub fn offer_one<R1, R2, M, A>(
    _o: OfferOne<R1, M, A>,
    channel: &mut TransportChannel,
) -> (M, A)
where
    M: Message + 'static + for<'a> PNetTcpMessage<'a>,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let message = M::from_pnet_tcp_packet(channel.recv());
    (message, A::new())
}

#[must_use]
pub fn select_one<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: M,
    channel: &mut TransportChannel,
) -> A
where
    M: Message + 'static + for<'a> PNetTcpMessage<'a>,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = message.to_pnet_tcp_packet();
    channel.send(packet);
    A::new()
}

pub fn offer_two<R1, R2, M1, M2, A1, A2>(
    _o: OfferTwo<R2, M1, M2, A1, A2>,
    channel: &mut TransportChannel,
    picker: Box<dyn Fn() -> bool>,
) -> Branch<(M1, A1), (M2, A2)>
where
    R1: Role,
    R2: Role,
    M1: Message + for<'a> PNetTcpMessage<'a>,
    M2: Message + for<'a> PNetTcpMessage<'a>,
    A1: Action,
    A2: Action,
{
    let choice = picker();
    match choice {
        true => Branch::Left((M1::from_pnet_tcp_packet(channel.recv()), A1::new())),
        false => Branch::Right((M2::from_pnet_tcp_packet(channel.recv()), A2::new())),
    }
}

#[must_use]
pub fn select_left<R1, R2, M1, M2, A1, A2>(
    _o: SelectTwo<R2, M1, M2, A1, A2>,
    channel: &mut TransportChannel,
    message: M1,
) -> A1
where
    R1: Role,
    R2: Role,
    M1: Message + for<'a> PNetTcpMessage<'a>,
    M2: Message + for<'a> PNetTcpMessage<'a>,
    A1: Action,
    A2: Action,
{
    channel.send(message.to_pnet_tcp_packet());
    A1::new()
}

#[must_use]
pub fn select_right<R1, R2, M1, M2, A1, A2>(
    _o: SelectTwo<R2, M1, M2, A1, A2>,
    channel: &mut TransportChannel,
    message: M2,
) -> A2
where
    R1: Role,
    R2: Role,
    M1: Message + for<'a> PNetTcpMessage<'a>,
    M2: Message + for<'a> PNetTcpMessage<'a>,
    A1: Action,
    A2: Action,
{
    channel.send(message.to_pnet_tcp_packet());
    A2::new()
}

mod tcp_messages;
mod pnet_channel;
mod crossbeam;