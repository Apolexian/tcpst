use std::marker::PhantomData;

use either::Either;

pub fn spawn<M, A: Action>() -> (Sender<M, A>, Reciever<M, A>) {
    todo!()
}

pub struct Sender<M, A: Action> {
    phantom: PhantomData<(M, A)>,
}

impl<M, A: Action> Sender<M, A> {
    pub fn send(&self, message: M, send: A) {
        todo!()
    }
}

pub struct Reciever<M, A: Action> {
    phantom: PhantomData<(M, A)>,
}

impl<M, A: Action> Reciever<M, A> {
    pub fn recv(&self) -> (M, A) {
        todo!()
    }
}

pub trait Action {
    type Dual: Action<Dual = Self>;
    fn new() -> (Self, Self::Dual)
    where
        Self: Sized;
}
pub struct Send<M, A>
where
    A: Action,
    A::Dual: Action,
{
    phantom: PhantomData<A>,
    channel: Sender<M, A::Dual>,
}

impl<M, A> Action for Send<M, A>
where
    A: Action,
{
    type Dual = Recv<M, A::Dual>;

    fn new() -> (Self, Self::Dual)
    where
        Self: Sized,
    {
        let (send, recv) = spawn::<M, A::Dual>();

        (
            Send {
                phantom: PhantomData::default(),
                channel: send,
            },
            Recv {
                phantom: PhantomData::default(),
                channel: recv,
            },
        )
    }
}

impl<M, A: Action> Drop for Send<M, A> {
    fn drop(&mut self) {
        std::mem::drop(self)
    }
}

pub struct Recv<M, A>
where
    A: Action,
{
    phantom: PhantomData<A>,
    channel: Reciever<M, A>,
}

impl<'a, M, A> Action for Recv<M, A>
where
    A: Action,
{
    type Dual = Send<M, A::Dual>;

    fn new() -> (Self, Self::Dual)
    where
        Self: Sized,
    {
        let (there, here) = Self::Dual::new();
        (here, there)
    }
}

impl<M, A: Action> Drop for Recv<M, A> {
    fn drop(&mut self) {
        std::mem::drop(self)
    }
}

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;

    fn new() -> (Self, Self::Dual)
    where
        Self: Sized,
    {
        (Terminate {}, Terminate {})
    }
}

pub fn send<M, A: Action>(message: M, send: Send<M, A>) -> A {
    let (this, that) = A::new();
    send.channel.send(message, that);
    this
}

pub fn recv<M, A: Action>(recv: Recv<M, A>) -> (M, A) {
    recv.channel.recv()
}

pub struct Offer<L, A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(L, A, O)>,
}

impl<L, A, O> Action for Offer<L, A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Choose<L, A::Dual, O::Dual>;

    fn new() -> (Self, Self::Dual)
    where
        Self: Sized,
    {
        (
            Offer {
                phantom: PhantomData::default(),
            },
            Choose {
                phantom: PhantomData::default(),
            },
        )
    }
}

pub enum Branch {
    Left,
    Right,
}

pub fn offer<A: Action, O: Action>(
    choice: Branch,
) -> Either<(A, <A as Action>::Dual), (O, <O as Action>::Dual)> {
    match choice {
        Branch::Left => Either::Left(A::new()),
        Branch::Right => Either::Right(O::new()),
    }
}

pub struct Choose<L, A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(L, A, O)>,
}

impl<L, A, O> Action for Choose<L, A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Offer<L, A::Dual, O::Dual>;

    fn new() -> (Self, Self::Dual)
    where
        Self: Sized,
    {
        (
            Choose {
                phantom: PhantomData::default(),
            },
            Offer {
                phantom: PhantomData::default(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tcp::{Abort, Segment},
        Action, Offer, Recv, Send, Terminate,
    };

    #[test]
    fn whatever() {}
}

mod proto;
mod tcp;
