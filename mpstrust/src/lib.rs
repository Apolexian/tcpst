use crossbeam::CrossBeamRoleChannel;
use std::marker::PhantomData;
use tcp_messages::{Ack, AckMessage, Syn, SynMessage};

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

pub struct OfferTwo<R1, R2, M1, M2, A1, A2>
where
    R1: Role,
    R2: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(R1, R2, M1, M2, A1, A2)>,
}

impl<R1, R2, M1, M2, A1, A2> Action for OfferTwo<R1, R2, M1, M2, A1, A2>
where
    R1: Role + std::marker::Send,
    R2: Role + std::marker::Send,
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

pub struct SelectTwo<R1, R2, M1, M2, A1, A2>
where
    R1: Role,
    R2: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(R1, R2, M1, M2, A1, A2)>,
}

impl<R1, R2, M1, M2, A1, A2> Action for SelectTwo<R1, R2, M1, M2, A1, A2>
where
    R1: Role + std::marker::Send,
    R2: Role + std::marker::Send,
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
    pub fn close(&self) {
        drop(self)
    }
}

#[must_use]
pub fn offer_one_ack<'a, R1, R2, M, A>(
    _o: OfferOne<R1, M, A>,
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> (Ack, A)
where
    M: AckMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let message = channel.recv.recv().unwrap();
    let ack = CrossBeamRoleChannel::<R1, R2>::to_ack_message(message);
    (ack, A::new())
}

#[must_use]
pub fn offer_one_syn<'a, R1, R2, M, A>(
    _o: OfferOne<R1, M, A>,
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> (Syn, A)
where
    M: SynMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let message = channel.recv.recv().unwrap();
    let syn = CrossBeamRoleChannel::<R1, R2>::to_syn_message(message);
    (syn, A::new())
}

#[must_use]
pub fn select_one_ack<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: Ack,
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> A
where
    M: AckMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = CrossBeamRoleChannel::<R1, R2>::from_ack_message(message);
    channel.send.send(packet).unwrap();
    A::new()
}

#[must_use]
pub fn select_one_syn<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: Syn,
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> A
where
    M: SynMessage + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = CrossBeamRoleChannel::<R1, R2>::from_syn_message(message);
    channel.send.send(packet).unwrap();
    A::new()
}

pub fn offer_two_ack_syn<R1, R2, R3, M1, M2, A1, A2>(
    _o: OfferTwo<R2, R3, M1, M2, A1, A2>,
    channel_one: CrossBeamRoleChannel<R2, R1>,
    channel_two: CrossBeamRoleChannel<R3, R1>,
    picker: Box<dyn Fn() -> bool>,
) -> Branch<(Ack, A1), (Syn, A2)>
where
    R1: Role,
    R2: Role,
    R3: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    let choice = picker();
    match choice {
        true => Branch::Left((
            CrossBeamRoleChannel::<R2, R1>::to_ack_message(channel_one.recv.recv().unwrap()),
            A1::new(),
        )),
        false => Branch::Right((
            CrossBeamRoleChannel::<R3, R1>::to_syn_message(channel_two.recv.recv().unwrap()),
            A2::new(),
        )),
    }
}

#[must_use]
pub fn select_ack_in_ack_syn<R1, R2, R3, M1, M2, A1, A2>(
    _o: SelectTwo<R2, R3, M1, M2, A1, A2>,
    channel: CrossBeamRoleChannel<R1, R2>,
    message: Ack,
) -> A1
where
    R1: Role,
    R2: Role,
    R3: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    channel
        .send
        .send(CrossBeamRoleChannel::<R1, R2>::from_ack_message(message))
        .unwrap();
    A1::new()
}

#[must_use]
pub fn select_syn_in_ack_syn<R1, R2, R3, M1, M2, A1, A2>(
    _o: SelectTwo<R2, R3, M1, M2, A1, A2>,
    channel: CrossBeamRoleChannel<R1, R2>,
    message: Syn,
) -> A2
where
    R1: Role,
    R2: Role,
    R3: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    channel
        .send
        .send(CrossBeamRoleChannel::<R1, R2>::from_syn_message(message))
        .unwrap();
    A2::new()
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{
        crossbeam::CrossBeamRoleChannel,
        offer_one_ack, offer_one_syn, select_one_ack, select_one_syn,
        tcp_messages::{Ack, Syn},
        Action, End, OfferOne, Role, SelectOne,
    };
    use crossbeam_channel::unbounded;

    #[derive(Clone)]
    struct RoleA {}
    impl Role for RoleA {}

    #[derive(Clone)]
    struct RoleB {}
    impl Role for RoleB {}

    struct RoleC {}
    impl Role for RoleC {}
    #[derive(Clone)]

    struct RoleD {}
    impl Role for RoleD {}

    #[test]
    fn simple_protocol() {
        // Create a channel that connects [`RoleB`] to [`RoleA`]
        // This channel will send from [`RoleB`] to [`RoleA`]
        // and recv from [`RoleB`] on [`RoleA`].
        let channels_a = unbounded();
        let channel = CrossBeamRoleChannel::<RoleB, RoleA>::new(channels_a.0, channels_a.1);
        // Declare the Session Type for the participants
        type LocalViewA = OfferOne<RoleB, Syn, OfferOne<RoleB, Ack, End>>;
        type LocalViewB = SelectOne<RoleA, Syn, SelectOne<RoleA, Ack, End>>;
        let a = LocalViewA::new();
        let b = LocalViewB::new();

        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let (_, cont) = offer_one_syn(a, &channel);
                let (_, cont) = offer_one_ack(cont, &channel);
                cont.close();
            });
            let thread_b = scope.spawn(|| {
                let cont = select_one_syn(b, Syn {}, &channel);
                let cont = select_one_ack(cont, Ack {}, &channel);
                cont.close();
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
        });
    }
}

mod crossbeam;
mod tcp_messages;
