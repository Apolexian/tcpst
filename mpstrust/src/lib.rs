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
        offer_one, select_one, Action, ChannelRecv, ChannelSend, End, Message, OfferOne, Role,
        RoleChannel, SelectOne,
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
}
