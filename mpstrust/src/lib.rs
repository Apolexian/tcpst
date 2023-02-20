use std::{marker::PhantomData, mem::MaybeUninit};

pub trait Action: Send {
    fn new() -> Self
    where
        Self: Sized;
}

pub struct ServerSystemClientSystemOfferOne<M, A>
where
    M: Message,
    A: Action,
{
    endpoint: MaybeUninit<Box<dyn ChannelRecv<M>>>,
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for ServerSystemClientSystemOfferOne<M, A>
where
    M: Message,
    A: Action,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        ServerSystemClientSystemOfferOne {
            endpoint: MaybeUninit::uninit(),
            phantom: PhantomData,
        }
    }
}

pub fn server_system_client_system_offer_one<M, A>(
    o: ServerSystemClientSystemOfferOne<M, A>,
) -> (M, A)
where
    M: Message + 'static,
    A: Action + 'static,
{
    unsafe {
        let endpoint = o.endpoint.assume_init();
        (endpoint.recv(), A::new())
    }
}

impl<M, A> ChannelAssociateRecv<M> for ServerSystemClientSystemOfferOne<M, A>
where
    M: Message,
    A: Action,
{
    fn associate_channel(&mut self, c: Box<dyn ChannelRecv<M>>) {
        self.endpoint = MaybeUninit::new(c);
    }
}

pub struct ClientSystemServerSystemSelectOne<M, A>
where
    M: Message,
    A: Action,
{
    endpoint: MaybeUninit<Box<dyn ChannelSend<M>>>,
    phantom: PhantomData<(M, A)>,
}

impl<M, A> Action for ClientSystemServerSystemSelectOne<M, A>
where
    M: Message,
    A: Action,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        ClientSystemServerSystemSelectOne {
            endpoint: MaybeUninit::uninit(),
            phantom: PhantomData,
        }
    }
}

pub fn client_system_server_system_select_one<M, A>(
    o: ClientSystemServerSystemSelectOne<M, A>,
    message: M,
) -> A
where
    M: Message + 'static,
    A: Action + 'static,
{
    unsafe {
        let endpoint = o.endpoint.assume_init();
        endpoint.send(message);
    }
    A::new()
}

impl<M, A> ChannelAssociateSend<M> for ClientSystemServerSystemSelectOne<M, A>
where
    M: Message,
    A: Action,
{
    fn associate_channel(&mut self, c: Box<dyn ChannelSend<M>>) {
        self.endpoint = MaybeUninit::new(c);
    }
}

pub trait Choice {}

pub trait ChannelRecv<M>: Send {
    fn recv(&self) -> M;
}

pub trait ChannelSend<M>: Send {
    fn send(&self, message: M);
}

pub trait ChannelAssociateRecv<M> {
    fn associate_channel(&mut self, c: Box<dyn ChannelRecv<M>>);
}

pub trait ChannelAssociateSend<M> {
    fn associate_channel(&mut self, c: Box<dyn ChannelSend<M>>);
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

#[cfg(test)]
mod tests {
    use std::thread;

    use crossbeam_channel::{unbounded, Receiver, Sender};

    use crate::{
        client_system_server_system_select_one, server_system_client_system_offer_one, Action,
        ChannelAssociateRecv, ChannelAssociateSend, ChannelRecv, ChannelSend,
        ClientSystemServerSystemSelectOne, End, Message, ServerSystemClientSystemOfferOne,
    };

    impl<M> ChannelSend<M> for Sender<M>
    where
        M: Message + Send,
    {
        fn send(&self, message: M) {
            self.send(message).unwrap();
        }
    }

    impl<M> ChannelRecv<M> for Receiver<M>
    where
        M: Message + Send,
    {
        fn recv(&self) -> M {
            self.recv().unwrap()
        }
    }

    impl Message for u32 {}

    #[test]
    fn simple_protocol() {
        type LocalViewA =
            ServerSystemClientSystemOfferOne<u32, ServerSystemClientSystemOfferOne<u32, End>>;
        type LocalViewB =
            ClientSystemServerSystemSelectOne<u32, ClientSystemServerSystemSelectOne<u32, End>>;
        let mut a = LocalViewA::new();
        let mut b = LocalViewB::new();
        let channels_a = unbounded();
        a.associate_channel(Box::new(channels_a.1.clone()));
        b.associate_channel(Box::new(channels_a.0.clone()));

        let thread_a = thread::spawn(move || {
            let (val, mut cont) = server_system_client_system_offer_one(a);
            assert_eq!(val, 10);
            cont.associate_channel(Box::new(channels_a.1.clone()));
            let (val, cont) = server_system_client_system_offer_one(cont);
            assert_eq!(val, 15);
            cont.close();
        });

        let thread_b = thread::spawn(move || {
            let mut cont = client_system_server_system_select_one(b, 10);
            cont.associate_channel(Box::new(channels_a.0.clone()));
            let cont = client_system_server_system_select_one(cont, 15);
            cont.close();
        });
        thread_a.join().unwrap();
        thread_b.join().unwrap();
    }
}
