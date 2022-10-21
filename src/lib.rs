#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::marker::PhantomData;

pub trait Message: PartialEq + Ord + Default {}

pub trait Action {
    type Dual;
    type Cont;

    fn get_cont(&self) -> Self::Cont;
    fn new() -> Self;
}

pub struct Send<M, A>
where
    M: Message,
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M: Message, A: Action> Send<M, A>
where
    <A as Action>::Dual: Action,
{
    fn emit(&self, message: M, emitter: Box<dyn Fn(M)>) -> A {
        (emitter)(message);
        self.get_cont()
    }
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
    <A as Action>::Cont: Action,
    O: Action,
    <O as Action>::Dual: Action,
    <O as Action>::Cont: Action,
{
    type Dual = Choose<A::Dual, O::Dual>;
    type Cont = (A::Cont, O::Cont);

    fn get_cont(&self) -> Self::Cont {
        (<A::Cont as Action>::new(), <O::Cont as Action>::new())
    }

    fn new() -> Self {
        Offer {
            phantom: PhantomData::default(),
        }
    }
}

pub struct Choose<A: Action, O: Action> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> Action for Choose<A, O>
where
    A: Action,
    <A as Action>::Dual: Action,
    <A as Action>::Cont: Action,
    O: Action,
    <O as Action>::Dual: Action,
    <O as Action>::Cont: Action,
{
    type Dual = Offer<A::Dual, O::Dual>;
    type Cont = (A::Cont, O::Cont);

    fn get_cont(&self) -> Self::Cont {
        (<A::Cont as Action>::new(), <O::Cont as Action>::new())
    }

    fn new() -> Self {
        Choose {
            phantom: PhantomData::default(),
        }
    }
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

pub struct Recv<M: Message, A: Action> {
    phantom: PhantomData<(M, A)>,
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

impl<M: Message, A: Action> Recv<M, A>
where
    <A as Action>::Dual: Action,
{
    fn emit<T: Sized>(&self, message: M, emitter: Box<dyn Fn(M) -> T>) -> (T, A) {
        let val = (emitter)(message);
        (val, self.get_cont())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        tcp::{Abort, Segment},
        Action, Recv, Send, Terminate,
    };

    #[test]
    fn whatever() {
        type Protocol = Send<Segment, Recv<Abort, Terminate>>;
        let protocol = Protocol::new();
        let msg = Segment::default();
        let continuation = protocol.emit(msg, Box::new(|_| return));
        let msg = Abort::default();
        let (val, _) = continuation.emit(msg, Box::new(|_| return 10_u16));
        println!("{:?}", val);

        // Offer
        // We would like something like Recv<Segment, Offer<(Label, Action), (Label, Action), ...>>
        // So using it would be something like
        //
        //    // server
        //    prot = protocol::new()
        //    cont = prot.emit()
        //    cont = cont.offer([(Label, Action)]);
        //    ...
        //
        //    // client
        //    client = protocol::Dual::new()
        //    cont = client.emit()
        //    cont = client.choose(Label)
        //    ...
        //
        //    .offer and .choose would basically have the internals of .recv/.send the label
        //    the server then matches on this and returns the expected continuation
        //    the client should already know what it thinks the continuation should be
        //    Problems: No variadic generics so how can I parametrise Offer<..(L, A)> ?
    }
}

mod proto;
mod tcp;
