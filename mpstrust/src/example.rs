use std::marker::PhantomData;

pub trait Action {
    fn new() -> Self;
}

struct RecvFromRoleA<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for RecvFromRoleA<M, A>
where
    A: Action,
{
    fn new() -> Self {
        RecvFromRoleA {
            phantom: PhantomData,
        }
    }
}

impl<M, A> RecvFromRoleA<M, A>
where
    A: Action,
{
    pub fn recv(self, emitter: Box<dyn Fn() -> M>) -> (M, A) {
        let message = emitter();
        (message, A::new())
    }
}

struct SendToRoleA<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> SendToRoleA<M, A>
where
    A: Action,
{
    pub fn send(self, message: M, emitter: Box<dyn Fn(M)>) -> A {
        emitter(message);
        A::new()
    }
}

impl<M, A> Action for SendToRoleA<M, A>
where
    A: Action,
{
    fn new() -> Self {
        SendToRoleA {
            phantom: PhantomData,
        }
    }
}

struct RecvFromRoleB<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for RecvFromRoleB<M, A>
where
    A: Action,
{
    fn new() -> Self {
        RecvFromRoleB {
            phantom: PhantomData,
        }
    }
}

impl<M, A> RecvFromRoleB<M, A>
where
    A: Action,
{
    pub fn recv(self, emitter: Box<dyn Fn() -> M>) -> (M, A) {
        let message = emitter();
        (message, A::new())
    }
}

struct SendToRoleB<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> SendToRoleB<M, A>
where
    A: Action,
{
    pub fn send(self, message: M, emitter: Box<dyn Fn(M)>) -> A {
        emitter(message);
        A::new()
    }
}

impl<M, A> Action for SendToRoleB<M, A>
where
    A: Action,
{
    fn new() -> Self {
        SendToRoleB {
            phantom: PhantomData,
        }
    }
}

struct RecvFromRoleC<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for RecvFromRoleC<M, A>
where
    A: Action,
{
    fn new() -> Self {
        RecvFromRoleC {
            phantom: PhantomData,
        }
    }
}

impl<M, A> RecvFromRoleC<M, A>
where
    A: Action,
{
    pub fn recv(self, emitter: Box<dyn Fn() -> M>) -> (M, A) {
        let message = emitter();
        (message, A::new())
    }
}

struct SendToRoleC<M, A>
where
    A: Action,
{
    phantom: PhantomData<(M, A)>,
}

impl<M, A> SendToRoleC<M, A>
where
    A: Action,
{
    pub fn send(self, message: M, emitter: Box<dyn Fn(M)>) -> A {
        emitter(message);
        A::new()
    }
}

impl<M, A> Action for SendToRoleC<M, A>
where
    A: Action,
{
    fn new() -> Self {
        SendToRoleC {
            phantom: PhantomData,
        }
    }
}

struct End {}

impl Action for End {
    fn new() -> Self {
        End {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, End, RecvFromRoleB, RecvFromRoleC, SendToRoleA, SendToRoleC};

    type C = SendToRoleA<u32, RecvFromRoleB<u32, End>>;
    type A = RecvFromRoleC<u32, RecvFromRoleB<u32, End>>;
    type B = SendToRoleC<u32, SendToRoleA<u32, End>>;

    #[test]
    fn it_works_2() {
        let context_a: A = Action::new();
        let context_b: B = Action::new();
        let context_C: C = Action::new();
    }
}
