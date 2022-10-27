use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use either::Either;

pub trait SendEndpoint<M>: std::marker::Send {
    fn send(&self, message: M);
}

pub trait RecvEndpoint<M>: std::marker::Send {
    fn recv(&self) -> M;
}

struct Channel<A: Action, M> {
    sender: Box<dyn SendEndpoint<M>>,
    receiver: Box<dyn RecvEndpoint<M>>,
    phantom: PhantomData<A>,
}

impl<A: Action, M> Channel<A, M> {
    #[must_use]
    pub fn new(sender: Box<dyn SendEndpoint<M>>, receiver: Box<dyn RecvEndpoint<M>>) -> Self {
        Channel {
            sender,
            receiver,
            phantom: PhantomData,
        }
    }
}

pub trait Action {
    type Dual: Action<Dual = Self>;
}
pub struct Send<M, A>
where
    A: Action,
    A::Dual: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for Send<M, A>
where
    A: Action,
{
    type Dual = Recv<M, A::Dual>;
}

impl<M, A: Action> Channel<Send<M, A>, M> {
    #[must_use]
    pub fn send(self, message: M) -> Channel<A, M> {
        self.sender.send(message);
        let pin = ManuallyDrop::new(self);
        unsafe {
            Channel {
                sender: ptr::read(&(pin).sender as *const _),
                receiver: ptr::read(&(pin).receiver as *const _),
                phantom: PhantomData,
            }
        }
    }
}

pub struct Recv<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<'a, M, A> Action for Recv<M, A>
where
    A: Action,
{
    type Dual = Send<M, A::Dual>;
}

impl<M, A: Action> Channel<Recv<M, A>, M> {
    #[must_use]
    pub fn recv(self) -> (M, Channel<A, M>) {
        let pin = ManuallyDrop::new(self);
        (pin.receiver.recv(), unsafe {
            Channel {
                sender: ptr::read(&(pin).sender as *const _),
                receiver: ptr::read(&(pin).receiver as *const _),
                phantom: PhantomData,
            }
        })
    }
}

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;
}

impl<M> Channel<Terminate, M> {
    pub fn close(self) {
        let pin = ManuallyDrop::new(self);
        let sender = unsafe { ptr::read(&(pin).sender as *const _) };
        let receiver = unsafe { ptr::read(&(pin).receiver as *const _) };

        drop(sender);
        drop(receiver);
    }
}

pub struct Choice<M, A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(M, A, O)>,
}

impl<M, A, O> Action for Choice<M, A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Choice<M, A::Dual, O::Dual>;
}

pub enum Branch {
    Left,
    Right,
}

/// This construct is semi-problematic to deal with over the network.
/// The idea for a normal offer/choose implementation is to implement these via send/recv.
/// When a client wishes to make a choice it sends an indication of which choice (a label to match on enums/ true-false)
/// via a channel and the server recieves. Hence, we can just match on this and return an Either, which can also be matched on.
/// Over the network we can't send this indication in a packet. We base our choice on the type of message sent via flags (or other enocded mechanisms).
/// So, the `offer` function for a server will simply require a function that provides this message - `message_emitter`.
/// We also need some function that casts this message to a left or right choice - `caster`.
/// So effectively we would:
///
/// 1) write a function that gets a packet and derives its type
/// 2) write a function that casts that type to left or right
/// 3) in the server code match on the return type
///
/// On the client side we only really need to proceed as whatever we want, we synchronise this choice with the server by sending a packet
/// with some message type. If the choice is not cast/derived correctly then we expect communication to desync and abort at the next stage (or later stages
/// once we realise).
/// So the client would just use the `choose` function as:
/// Pass a choice left/right, get back an `Either` of these and then use the corresponding side, i.e. .left() or .right()
impl<A: Action, O: Action, M, T> Channel<Choice<M, A, O>, T> {
    #[must_use]
    pub fn offer(
        &self,
        message_emitter: Box<dyn Fn() -> M>,
        caster: Box<dyn Fn(M) -> Branch>,
    ) -> Either<Channel<A, T>, Channel<O, T>> {
        let message = message_emitter();
        let choice = caster(message);
        unsafe {
            let pin = ManuallyDrop::new(self);
            match choice {
                Branch::Left => Either::Left(Channel::new(
                    ptr::read(&(pin).sender),
                    ptr::read(&(pin).receiver),
                )),
                Branch::Right => Either::Right(Channel::new(
                    ptr::read(&(pin).sender),
                    ptr::read(&(pin).receiver),
                )),
            }
        }
    }

    pub fn choose(&self, choice: Branch) -> Either<Channel<A, T>, Channel<O, T>> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            match choice {
                Branch::Left => Either::Left(Channel::new(
                    ptr::read(&(pin).sender),
                    ptr::read(&(pin).receiver),
                )),
                Branch::Right => Either::Right(Channel::new(
                    ptr::read(&(pin).sender),
                    ptr::read(&(pin).receiver),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crossbeam_channel::{unbounded, Receiver, Sender};

    use crate::{Action, Channel, Recv, RecvEndpoint, SendEndpoint, Terminate};

    impl<M: std::marker::Send> SendEndpoint<M> for Sender<M> {
        fn send(&self, message: M) {
            self.send(message).unwrap();
        }
    }

    impl<M: std::marker::Send> RecvEndpoint<M> for Receiver<M> {
        fn recv(&self) -> M {
            self.recv().unwrap()
        }
    }

    #[test]
    fn test_client_send_server_recv_terminate() {
        type Protocol = Recv<u16, Recv<u16, Terminate>>;
        type Dual = <Protocol as Action>::Dual;
        let (r1, s1) = unbounded();
        let (r2, s2) = unbounded();

        {
            thread::spawn(|| {
                let channel_client: Channel<Dual, u16> = Channel::new(Box::new(r2), Box::new(s1));
                let cont = channel_client.send(10);
                println!("sent");
                let cont = cont.send(15);
                println!("sent");
                cont.close();
            })
        };
        {
            thread::spawn(|| {
                let channel_server: Channel<Protocol, u16> =
                    Channel::new(Box::new(r1), Box::new(s2));
                let (val, cont) = channel_server.recv();
                println!("{:?}", val);
                let (val, cont) = cont.recv();
                println!("{:?}", val);
                cont.close();
            })
            .join()
            .unwrap();
        };
    }
}

mod proto;
mod tcp;
