#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::marker::PhantomData;

/// TODO:
/// 1) Need some sort of way to represent a session type, i.e. encode the TCP state machine in types
///
/// 2) Need some way to decide if the path we are taking is acceptable by the session type
///
/// 3) Need to decide what happens if the path is not acceptable
///
/// 4) Need to decide how/what the communication between the client and server consists of
///

pub trait Message: PartialEq + Ord + Default {}

pub trait Action {
    type Dual;
    type Cont;

    fn get_cont(&self) -> Self::Cont;
    fn new() -> Self;
}

pub struct Sender<M: Message, A: Action> {
    phantom: PhantomData<A>,
    dispatch: Box<dyn Fn(M)>,
}

impl<M, A> Sender<M, A>
where
    M: Message,
    A: Action,
{
    pub fn new(dispatch: Box<dyn Fn(M)>) -> Self {
        Sender {
            phantom: PhantomData::default(),
            dispatch,
        }
    }

    pub fn send<S: Action + Action<Cont = S>>(&self, message: M, action: S) -> S
    where
        <A as Action>::Cont: Action,
    {
        (self.dispatch)(message);
        action.get_cont()
    }
}

pub struct Send<M: Message, A: Action> {
    phantom: PhantomData<(M, A)>,
}

impl<M: Message, A> Action for Send<M, A>
where
    A: Action,
    <A as Action>::Dual: Action,
{
    type Dual = Recv<M, A::Dual>;
    type Cont = A;

    fn new() -> Self {
        Send {
            phantom: PhantomData::default(),
        }
    }

    fn get_cont(&self) -> Self::Cont {
        A::new()
    }
}

pub struct Offer<A: Action, O: Action> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O: Action> Action for Offer<A, O>
where
    A: Action,
    <A as Action>::Dual: Action,
    O: Action,
    <O as Action>::Dual: Action,
{
    type Dual = Choose<A::Dual, O::Dual>;
    type Cont = A;

    fn get_cont(&self) -> Self::Cont {
        todo!()
    }

    fn new() -> Self {
        todo!()
    }
}

pub struct Choose<A: Action, O: Action> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> Action for Choose<A, O>
where
    A: Action,
    <A as Action>::Dual: Action,
    O: Action,
    <O as Action>::Dual: Action,
{
    type Dual = Offer<A::Dual, O::Dual>;
    type Cont = A;

    fn get_cont(&self) -> Self::Cont {
        todo!()
    }

    fn new() -> Self {
        todo!()
    }
}

pub struct Recv<M: Message, A: Action> {
    phantom: PhantomData<(M, A)>,
}

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;
    type Cont = Terminate;

    fn get_cont(&self) -> Self::Cont {
        todo!()
    }

    fn new() -> Self {
        todo!()
    }
}

impl<M: Message, A> Action for Recv<M, A>
where
    A: Action,
    <A as Action>::Dual: Action,
{
    type Dual = Send<M, A::Dual>;
    type Cont = A;

    fn get_cont(&self) -> Self::Cont {
        A::new()
    }

    fn new() -> Self {
        Recv {
            phantom: PhantomData::default(),
        }
    }
}

pub enum Branch<L: Action, R: Action> {
    Left(L),
    Right(R),
}

#[cfg(test)]
mod tests {
    use crate::{
        tcp::{Abort, Segment},
        Recv, Send, Sender, Terminate,
    };

    #[test]
    fn whatever() {
        type Example = Send<Segment, Recv<Abort, Terminate>>;

        let s: Sender<Segment, Example> = Sender::new(Box::new(|_| {}));
    }
}

mod proto;
mod tcp;
