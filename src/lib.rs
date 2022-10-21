use std::marker::PhantomData;

use either::Either;

pub fn spawn<M, A: Action>() -> (Sender<A>, Reciever<A>) {
    todo!()
}

pub struct Sender<A: Action> {
    phantom: PhantomData<A>,
}

impl<A: Action> Sender<A> {
    pub fn send<M>(&self, message: M, send: A) {
        todo!()
    }
}

pub struct Reciever<A: Action> {
    phantom: PhantomData<A>,
}

impl<A: Action> Reciever<A> {
    pub fn recv(&self) -> A {
        todo!()
    }
}
pub struct Channel<A: Action>(Sender<A>, Reciever<A>);

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

impl<M, A: Action> Channel<Send<M, A>> {
    pub fn send(&self, message: M) -> Channel<A> {
        todo!()
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

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;
}

pub struct Offer<A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(A, O)>,
}

impl<A, O> Action for Offer<A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Choose<A::Dual, O::Dual>;
}

pub enum Branch {
    Left,
    Right,
}

pub struct Choose<A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(A, O)>,
}

impl<A, O> Action for Choose<A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Offer<A::Dual, O::Dual>;
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
