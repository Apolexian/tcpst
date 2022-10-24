use std::{marker::PhantomData, mem, ptr};

use either::Either;

struct Sender<A: Action, M> {
    phantom: PhantomData<A>,
    emitter: Box<dyn Fn(M)>,
}

impl<A: Action, M> Sender<A, M> {
    pub fn send(&self, message: M) {
        (self.emitter)(message);
    }
}

struct Reciever<A: Action, M> {
    phantom: PhantomData<A>,
    emitter: Box<dyn Fn() -> M>,
}

impl<A: Action, M> Reciever<A, M> {
    pub fn recv(&self) -> M {
        (self.emitter)()
    }
}

struct Channel<A: Action, M> {
    sender: Sender<A, M>,
    reciever: Reciever<A::Dual, M>,
}

impl<A: Action, M> Channel<A, M> {
    #[must_use]
    pub fn new(send_emitter: Box<dyn Fn(M)>, recv_emitter: Box<dyn Fn() -> M>) -> Self {
        Channel {
            sender: Sender {
                phantom: PhantomData::default(),
                emitter: send_emitter,
            },
            reciever: Reciever {
                phantom: PhantomData::default(),
                emitter: recv_emitter,
            },
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

impl<M, A: Action> Drop for Send<M, A> {
    fn drop(&mut self) {
        std::mem::drop(self)
    }
}

impl<M, A: Action> Channel<Send<M, A>, M> {
    #[must_use]
    pub fn send(&self, message: M) -> Channel<A, M> {
        self.sender.send(message);
        unsafe {
            Channel {
                sender: Sender {
                    phantom: PhantomData::default(),
                    emitter: ptr::read(&(self).sender.emitter as *const _),
                },
                reciever: Reciever {
                    phantom: PhantomData::default(),
                    emitter: ptr::read(&(self).reciever.emitter as *const _),
                },
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

impl<M, A: Action> Drop for Recv<M, A> {
    fn drop(&mut self) {
        std::mem::drop(self)
    }
}

impl<M, A: Action> Channel<Recv<M, A>, M> {
    #[must_use]
    pub fn recv(&self) -> (M, Channel<A, M>) {
        ((self.reciever.emitter)(), unsafe {
            Channel {
                sender: Sender {
                    phantom: PhantomData::default(),
                    emitter: ptr::read(&(self).sender.emitter as *const _),
                },
                reciever: Reciever {
                    phantom: PhantomData::default(),
                    emitter: ptr::read(&(self).reciever.emitter as *const _),
                },
            }
        })
    }
}

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;
}

impl<M> Channel<Terminate, M> {
    pub fn close(&self) {
        mem::drop(self);
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
            match choice {
                Branch::Left => Either::Left(Channel::new(
                    ptr::read(&(self).sender.emitter as *const _),
                    ptr::read(&(self).reciever.emitter as *const _),
                )),
                Branch::Right => Either::Right(Channel::new(
                    ptr::read(&(self).sender.emitter as *const _),
                    ptr::read(&(self).reciever.emitter as *const _),
                )),
            }
        }
    }

    pub fn choose(&self, choice: Branch) -> Either<Channel<A, T>, Channel<O, T>> {
        unsafe {
            match choice {
                Branch::Left => Either::Left(Channel::new(
                    ptr::read(&(self).sender.emitter as *const _),
                    ptr::read(&(self).reciever.emitter as *const _),
                )),
                Branch::Right => Either::Right(Channel::new(
                    ptr::read(&(self).sender.emitter as *const _),
                    ptr::read(&(self).reciever.emitter as *const _),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Branch, Channel, Choice, Send, Terminate};

    #[test]
    fn whatever() {
        type Protocol = Send<u16, Send<u16, Terminate>>;

        let send_emitter = Box::new(|x: u16| eprintln!("{:?}", x));
        let recv_emitter = Box::new(|| return 10_u16);

        let channel: Channel<Protocol, u16> = Channel::new(send_emitter, recv_emitter);

        let a = channel.send(10);
        let b = a.send(10);
        b.close();
    }

    #[test]
    fn whatever2() {
        type Protocol = Choice<bool, Send<u16, Terminate>, Terminate>;
        let send_emitter = Box::new(|x: u16| eprintln!("{:?}", x));
        let recv_emitter = Box::new(|| return 10_u16);
        let picker = Box::new(|choice| match choice {
            true => Branch::Left,
            false => Branch::Right,
        });
        let message_emitter = Box::new(|| true);
        let channel: Channel<Protocol, u16> = Channel::new(send_emitter, recv_emitter);

        match channel.offer(message_emitter, picker) {
            either::Either::Left(s) => {
                let cont = s.send(10);
                cont.close();
            }
            either::Either::Right(t) => {
                t.close();
            }
        }
    }
}

mod proto;
mod tcp;
