use crossbeam::CrossBeamRoleChannel;
use std::marker::PhantomData;

// Supporting traits

pub trait Action: Send {
    fn new() -> Self
    where
        Self: Sized;
}

pub trait Role {}

pub trait Message: Send {
    fn from_slice(slice: Vec<u8>) -> Self;
    fn to_slice(&self) -> Vec<u8>;
}

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
    pub fn close<R1, R2>(&self, channel: CrossBeamRoleChannel<R1, R2>)
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
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> (M, A)
where
    M: Message + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let message = channel.recv.recv().unwrap();
    let message = M::from_slice(message);
    (message, A::new())
}

#[must_use]
pub fn select_one<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: M,
    channel: &CrossBeamRoleChannel<R1, R2>,
) -> A
where
    M: Message + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    let packet = message.to_slice();
    channel.send.send(packet).unwrap();
    A::new()
}

pub fn offer_two<R1, R2, M1, M2, A1, A2>(
    _o: OfferTwo<R2, M1, M2, A1, A2>,
    channel: CrossBeamRoleChannel<R1, R2>,
    picker: Box<dyn Fn() -> bool>,
) -> Branch<(M1, A1), (M2, A2)>
where
    R1: Role,
    R2: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    let choice = picker();
    match choice {
        true => Branch::Left((M1::from_slice(channel.recv.recv().unwrap()), A1::new())),
        false => Branch::Right((M2::from_slice(channel.recv.recv().unwrap()), A2::new())),
    }
}

#[must_use]
pub fn select_left<R1, R2, M1, M2, A1, A2>(
    _o: SelectTwo<R2, M1, M2, A1, A2>,
    channel: CrossBeamRoleChannel<R1, R2>,
    message: M1,
) -> A1
where
    R1: Role,
    R2: Role,
    M1: Message,
    M2: Message,
    A1: Action,
    A2: Action,
{
    channel.send.send(message.to_slice()).unwrap();
    A1::new()
}

#[must_use]
pub fn select_right<R1, R2, R3, M1, M2, A1, A2>(
    _o: SelectTwo<R2, M1, M2, A1, A2>,
    channel: CrossBeamRoleChannel<R1, R2>,
    message: M2,
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
    channel.send.send(message.to_slice()).unwrap();
    A2::new()
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{
        crossbeam::CrossBeamRoleChannel,
        offer_one, select_one,
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
        let channel_clone = channel.clone();
        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let (_, cont) = offer_one(a, &channel_clone);
                let (_, cont) = offer_one(cont, &channel_clone);
                cont.close(channel_clone);
            });
            let thread_b = scope.spawn(|| {
                let cont = select_one(b, Syn {}, &channel);
                let cont = select_one(cont, Ack {}, &channel);
                cont.close(channel);
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
        });
    }
}

mod crossbeam;
mod tcp_messages;
