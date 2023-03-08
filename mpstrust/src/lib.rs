use crossbeam_channel::{Receiver, Sender};
use std::marker::PhantomData;

pub trait Action: Send {
    fn new() -> Self
    where
        Self: Sized;
}

pub trait Role {}

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

#[must_use]
pub fn offer_one<R1, R2, M, MS, A>(
    _o: OfferOne<R1, MS, A>,
    channel: &RoleChannel<R1, R2, M>,
) -> (M, A)
where
    M: Message + 'static,
    MS: Message,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    (channel.recv.recv().unwrap(), A::new())
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

pub fn offer_two<R1, R2, R3, M1, M2, A1, A2>(
    _o: OfferTwo<R2, R3, M1, M2, A1, A2>,
    channel_one: &RoleChannel<R2, R1, M1>,
    channel_two: &RoleChannel<R3, R1, M2>,
    picker: Box<dyn Fn() -> bool>,
) -> Branch<(M1, A1), (M2, A2)>
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
        true => Branch::Left((channel_one.recv.recv().unwrap(), A1::new())),
        false => Branch::Right((channel_two.recv.recv().unwrap(), A2::new())),
    }
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

#[must_use]
pub fn choose_left<R1, R2, R3, M1, M2, A1, A2>(
    _o: SelectTwo<R2, R3, M1, M2, A1, A2>,
    channel: &RoleChannel<R1, R2, M1>,
    message: M1,
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
    channel.send.send(message).unwrap();
    A1::new()
}

#[must_use]
pub fn choose_right<R1, R2, R3, M1, M2, A1, A2>(
    _o: SelectTwo<R2, R3, M1, M2, A1, A2>,
    channel: &RoleChannel<R1, R2, M2>,
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
    channel.send.send(message).unwrap();
    A2::new()
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

#[must_use]
pub fn select_one<R1, R2, M, A>(
    _o: SelectOne<R2, M, A>,
    message: M,
    channel: &RoleChannel<R1, R2, M>,
) -> A
where
    M: Message + 'static,
    A: Action + 'static,
    R1: Role,
    R2: Role,
{
    channel.send.send(message).unwrap();
    A::new()
}

pub trait Choice {}

pub trait ChannelRecv<R: Role, M>: Send {
    fn recv(&self) -> M;
}

pub trait ChannelSend<R: Role, M>: Send {
    fn send(&self, message: M);
}

pub trait Message: Send {}

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

#[derive(Clone)]
pub struct RoleChannel<R1: Role, R2: Role, M: Message> {
    send: Sender<M>,
    recv: Receiver<M>,
    phantom: PhantomData<(R1, R2)>,
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, sync::Arc, thread};

    use crossbeam_channel::{unbounded, Receiver, Sender};

    use crate::{
        choose_left, choose_right, offer_one, offer_two, select_one, Action, ChannelRecv,
        ChannelSend, End, Message, OfferOne, OfferTwo, Role, RoleChannel, SelectOne, SelectTwo,
    };

    impl<R, M> ChannelSend<R, M> for Sender<M>
    where
        M: Message + Send,
        R: Role,
    {
        fn send(&self, message: M) {
            self.send(message).unwrap();
        }
    }

    impl<R, M> ChannelRecv<R, M> for Receiver<M>
    where
        M: Message + Send,
        R: Role,
    {
        fn recv(&self) -> M {
            self.recv().unwrap()
        }
    }

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

    struct SegAckSet {}
    impl Message for SegAckSet {}
    struct SegSynAckSet {}
    impl Message for SegSynAckSet {}

    enum TcpMessage {
        SegAckSet(SegAckSet),
        SegSynAckSet(SegSynAckSet),
    }

    impl TcpMessage {
        /// Returns `true` if the tcp message is [`SegAckSet`].
        ///
        /// [`SegAckSet`]: TcpMessage::SegAckSet
        #[must_use]
        fn is_seg_ack_set(&self) -> bool {
            matches!(self, Self::SegAckSet(..))
        }

        /// Returns `true` if the tcp message is [`SegSynAckSet`].
        ///
        /// [`SegSynAckSet`]: TcpMessage::SegSynAckSet
        #[must_use]
        fn is_seg_syn_ack_set(&self) -> bool {
            matches!(self, Self::SegSynAckSet(..))
        }
    }

    impl Message for TcpMessage {}

    #[test]
    fn simple_protocol() {
        // Create a channel that connects [`RoleB`] to [`RoleA`]
        // This channel will send from [`RoleB`] to [`RoleA`]
        // and recv from [`RoleB`] on [`RoleA`].
        let channels_a = unbounded();
        let channel = Arc::new(RoleChannel::<RoleB, RoleA, TcpMessage> {
            send: channels_a.0,
            recv: channels_a.1,
            phantom: PhantomData::default(),
        });
        // Declare the Session Type for the participants
        type LocalViewA = OfferOne<RoleB, SegAckSet, OfferOne<RoleB, SegSynAckSet, End>>;
        type LocalViewB = SelectOne<RoleA, TcpMessage, SelectOne<RoleA, TcpMessage, End>>;
        let a = LocalViewA::new();
        let b = LocalViewB::new();

        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let (val, cont) = offer_one(a, &channel.clone());
                assert!(val.is_seg_ack_set());
                let (val, cont) = offer_one(cont, &channel.clone());
                assert!(val.is_seg_syn_ack_set());
                cont.close();
            });
            let thread_b = scope.spawn(|| {
                let cont = select_one(b, TcpMessage::SegAckSet(SegAckSet {}), &channel.clone());
                let cont = select_one(
                    cont,
                    TcpMessage::SegSynAckSet(SegSynAckSet {}),
                    &channel.clone(),
                );
                cont.close();
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
        });
    }

    #[test]
    fn choice_protocol_left() {
        // Create a channel that connects [`RoleB`] to [`RoleA`]
        // This channel will send from [`RoleB`] to [`RoleA`]
        // and recv from [`RoleB`] on [`RoleA`].
        let channels_a = unbounded();
        let channel = Arc::new(RoleChannel::<RoleB, RoleA, TcpMessage> {
            send: channels_a.0,
            recv: channels_a.1,
            phantom: PhantomData::default(),
        });
        // Declare the Session Type for the participants
        type LocalViewA =
            OfferTwo<RoleB, RoleB, TcpMessage, TcpMessage, OfferOne<RoleB, SegSynAckSet, End>, End>;
        type LocalViewB =
            SelectTwo<RoleA, RoleA, TcpMessage, TcpMessage, SelectOne<RoleA, TcpMessage, End>, End>;
        let a = LocalViewA::new();
        let b = LocalViewB::new();

        fn picker() -> bool {
            true
        }

        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let variants = offer_two(a, &channel.clone(), &channel.clone(), Box::new(picker));
                match variants {
                    crate::Branch::Left((val, cont)) => {
                        assert!(val.is_seg_ack_set());
                        let (val, cont) = offer_one(cont, &channel.clone());
                        assert!(val.is_seg_syn_ack_set());
                        cont.close();
                    }
                    crate::Branch::Right((val, cont)) => {
                        assert!(val.is_seg_syn_ack_set());
                        cont.close();
                    }
                }
            });
            let thread_b = scope.spawn(|| {
                let cont = choose_left(b, &channel.clone(), TcpMessage::SegAckSet(SegAckSet {}));
                let cont = select_one(
                    cont,
                    TcpMessage::SegSynAckSet(SegSynAckSet {}),
                    &channel.clone(),
                );
                cont.close();
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
        });
    }

    #[test]
    fn choice_protocol_right() {
        // Create a channel that connects [`RoleB`] to [`RoleA`]
        // This channel will send from [`RoleB`] to [`RoleA`]
        // and recv from [`RoleB`] on [`RoleA`].
        let channels_a = unbounded();
        let channel = Arc::new(RoleChannel::<RoleB, RoleA, TcpMessage> {
            send: channels_a.0,
            recv: channels_a.1,
            phantom: PhantomData::default(),
        });
        // Declare the Session Type for the participants
        type LocalViewA =
            OfferTwo<RoleB, RoleB, TcpMessage, TcpMessage, OfferOne<RoleB, SegSynAckSet, End>, End>;
        type LocalViewB =
            SelectTwo<RoleA, RoleA, TcpMessage, TcpMessage, SelectOne<RoleA, TcpMessage, End>, End>;
        let a = LocalViewA::new();
        let b = LocalViewB::new();

        fn picker() -> bool {
            false
        }

        thread::scope(|scope| {
            let thread_a = scope.spawn(|| {
                let variants = offer_two(a, &channel.clone(), &channel.clone(), Box::new(picker));
                match variants {
                    crate::Branch::Left((val, cont)) => {
                        assert!(val.is_seg_ack_set());
                        let (val, cont) = offer_one(cont, &channel.clone());
                        assert!(val.is_seg_syn_ack_set());
                        cont.close();
                    }
                    crate::Branch::Right((val, cont)) => {
                        assert!(val.is_seg_ack_set());
                        cont.close();
                    }
                }
            });
            let thread_b = scope.spawn(|| {
                let cont = choose_right(b, &channel.clone(), TcpMessage::SegAckSet(SegAckSet {}));
                cont.close();
            });
            thread_a.join().unwrap();
            thread_b.join().unwrap();
        });
    }
}

mod nic;