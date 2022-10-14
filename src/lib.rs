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
}

pub struct Recv<M: Message, A: Action> {
    phantom: PhantomData<(M, A)>,
}

pub struct Terminate {}

impl Action for Terminate {
    type Dual = Terminate;
    type Cont = Terminate;
}

impl<M: Message, A> Action for Recv<M, A>
where
    A: Action,
    <A as Action>::Dual: Action,
{
    type Dual = Send<M, A::Dual>;
    type Cont = A;
}

#[cfg(test)]
mod tests {
    use crate::{
        tcp::{Abort, Segment},
        Action, Offer, Recv, Send, Terminate,
    };

    #[test]
    fn whatever() {
        type Example = Offer<Send<Segment, Terminate>, Recv<Abort, Terminate>>;
        type ExampleDual =
            <Offer<Send<Segment, Terminate>, Recv<Abort, Terminate>> as Action>::Dual;
    }
}

mod proto;
mod tcp;
