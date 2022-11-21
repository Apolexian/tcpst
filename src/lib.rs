use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

pub trait SendEndpoint<M>: std::marker::Send {
    fn send(&self, message: M);
}

pub trait RecvEndpoint<M>: std::marker::Send {
    fn recv(&self) -> M;
}

struct Channel<A, M>
where
    A: Action,
{
    sender: Box<dyn SendEndpoint<M>>,
    receiver: Box<dyn RecvEndpoint<M>>,
    phantom: PhantomData<A>,
}

impl<A, M> Channel<A, M>
where
    A: Action,
{
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

pub trait Reducable: Action {
    type Reduced: Action;
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

impl<M, A: Action> Reducable for Send<M, A> {
    type Reduced = Self;
}

impl<M, A, T> Channel<Send<T, A>, M>
where
    A: Action,
{
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

impl<M, A> Action for Recv<M, A>
where
    A: Action,
{
    type Dual = Send<M, A::Dual>;
}

impl<M, A> Reducable for Recv<M, A>
where
    A: Action,
{
    type Reduced = Self;
}

impl<M, A, T> Channel<Recv<T, A>, M>
where
    A: Action,
{
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

impl Reducable for Terminate {
    type Reduced = Self;
}

impl<M> Channel<Terminate, M> {
    pub fn close(self) {
        drop(self);
    }
}

pub struct Offer<M, A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(M, A, O)>,
}

impl<M, A, O> Action for Offer<M, A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Choose<M, A::Dual, O::Dual>;
}

impl<M, A, O> Reducable for Offer<M, A, O>
where
    A: Action,
    O: Action,
{
    type Reduced = Self;
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

/// This construct is semi-problematic to deal with over the network.
/// The idea for a normal offer/choose implementation is to implement these via send/recv.
/// When a client wishes to make a choice it sends an indication of which choice (a label to match on enums/ true-false)
/// via a channel and the server recieves. Hence, we can just match on this and return an Either, which can also be matched on.
/// Over the network we can't send this indication in a packet. We base our choice on the type of message sent via flags (or other enocded mechanisms).
/// So, the `offer` function for a server will simply require a function that provides this message and casts it to a choice.
/// If we want to skip nested branches then we can just pass a closure that picks the direction to choose.
/// So effectively we would write a function that gets a packet, casts that to a choice of either left (true) or right (false)
///
/// On the client side we only really need to proceed as whatever we want, we synchronise this choice with the server by sending a packet
/// with some message type. If the choice is not cast/derived correctly then we expect communication to desync and abort at the next stage (or later stages
/// once we realise).
/// So the client would just use the corresponding choose function `choose_left` or `choose_right`
impl<A: Action, O: Action, M, T> Channel<Offer<M, A, O>, T> {
    #[must_use]
    pub fn offer(self, choice: Box<dyn Fn() -> bool>) -> Branch<Channel<A, T>, Channel<O, T>> {
        let choice = choice();
        unsafe {
            let pin = ManuallyDrop::new(self);
            let sender = ptr::read(&(pin).sender);
            let receiver = ptr::read(&(pin).receiver);
            if choice {
                Branch::Left(Channel::new(sender, receiver))
            } else {
                Branch::Right(Channel::new(sender, receiver))
            }
        }
    }
}

pub struct Choose<M, A, O>
where
    A: Action,
    O: Action,
{
    phantom: PhantomData<(M, A, O)>,
}

impl<M, A, O> Action for Choose<M, A, O>
where
    A: Action,
    O: Action,
{
    type Dual = Offer<M, A::Dual, O::Dual>;
}

impl<M, A, O> Reducable for Choose<M, A, O>
where
    A: Action,
    O: Action,
{
    type Reduced = Self;
}

impl<A: Action, O: Action, M, T> Channel<Choose<M, A, O>, T> {
    pub fn choose_left(self) -> Channel<A, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_right(self) -> Channel<O, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }
}

/// `F` - emitter function
pub struct Tau<A>
where
    A: Action,
{
    phantom: PhantomData<A>,
}

impl<A> Action for Tau<A>
where
    A: Action,
{
    type Dual = Self;
}

impl<A> Reducable for Tau<A>
where
    A: Reducable,
{
    type Reduced = A::Reduced;
}

impl<A, M> Channel<Tau<A>, M>
where
    A: Action,
{
    fn do_action<T>(self, mut action: Box<dyn FnMut(T)>, arg: T) -> Channel<A, M> {
        action(arg);
        let pin = ManuallyDrop::new(self);
        unsafe { Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver)) }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crossbeam_channel::{unbounded, Receiver, Sender};

    use crate::{
        Action, Branch, Channel, Offer, Recv, RecvEndpoint, Reducable, Send, SendEndpoint, Tau,
        Terminate,
    };

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
    fn test_multiplication_server() {
        type Protocol = Recv<u16, Recv<u16, Send<u16, Terminate>>>;
        type Dual = <Protocol as Action>::Dual;
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();

        {
            thread::spawn(|| {
                let num1 = 10_u16;
                let num2 = 2_u16;
                let channel_client: Channel<Dual, u16> = Channel::new(Box::new(s2), Box::new(r1));
                let cont = channel_client.send(num1);
                let cont = cont.send(num2);
                let (answer, cont) = cont.recv();
                assert_eq!(num1 * num2, answer);
                cont.close();
            })
        };
        {
            thread::spawn(|| {
                let channel_server: Channel<Protocol, u16> =
                    Channel::new(Box::new(s1), Box::new(r2));
                let (val1, cont) = channel_server.recv();
                let (val2, cont) = cont.recv();
                let answer = val1 * val2;
                let cont = cont.send(answer);
                cont.close();
            })
            .join()
            .unwrap();
        };
    }

    #[test]
    fn test_choice_simple() {
        type Protocol = Offer<u16, Terminate, Terminate>;
        type Dual = <Protocol as Action>::Dual;
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        {
            thread::spawn(|| {
                let channel_client: Channel<Dual, u16> = Channel::new(Box::new(s2), Box::new(r1));
                let cont = channel_client.choose_right();
                cont.close();
            })
        };

        {
            thread::spawn(|| {
                let channel_server: Channel<Protocol, u16> =
                    Channel::new(Box::new(s1), Box::new(r2));
                let cont = channel_server.offer(Box::new(Box::new(|| false)));
                match cont {
                    Branch::Left(cont) => {
                        cont.close();
                    }
                    Branch::Right(cont) => {
                        cont.close();
                    }
                }
            })
            .join()
            .unwrap();
        };
    }

    #[test]
    fn test_add_two_or_three_numbers() {
        type Protocol = Offer<
            bool,
            Recv<u16, Recv<u16, Send<u16, Terminate>>>,
            Recv<u16, Recv<u16, Recv<u16, Send<u16, Terminate>>>>,
        >;
        type Dual = <Protocol as Action>::Dual;
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();

        pub fn picker_right() -> bool {
            false
        }

        {
            thread::spawn(|| {
                let num1 = 10_u16;
                let num2 = 2_u16;
                let num3 = 3_u16;
                let channel_client: Channel<Dual, u16> = Channel::new(Box::new(s2), Box::new(r1));
                let cont = channel_client.choose_right();
                let cont = cont.send(num1);
                let cont = cont.send(num2);
                let cont = cont.send(num3);
                let (answer, cont) = cont.recv();
                assert_eq!(num1 + num2 + num3, answer);
                cont.close();
            })
        };

        {
            thread::spawn(|| {
                let channel_server: Channel<Protocol, u16> =
                    Channel::new(Box::new(s1), Box::new(r2));
                let cont = channel_server.offer(Box::new(picker_right));
                match cont {
                    Branch::Left(cont) => {
                        let (val1, cont) = cont.recv();
                        let (val2, cont) = cont.recv();
                        let val = val1 + val2;
                        let cont = cont.send(val);
                        cont.close();
                    }
                    Branch::Right(cont) => {
                        let (val1, cont) = cont.recv();
                        let (val2, cont) = cont.recv();
                        let (val3, cont) = cont.recv();
                        let val = val1 + val2 + val3;
                        let cont = cont.send(val);
                        cont.close();
                    }
                }
            })
            .join()
            .unwrap();
        };
    }

    #[test]
    fn test_simple_tau() {
        type Protocol = Tau<Terminate>;
        type Reduced = <Protocol as Reducable>::Reduced;
        type Client = <Reduced as Action>::Dual;

        fn dummy(s: String) {
            assert_eq!(s, "hello")
        }

        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        {
            thread::spawn(|| {
                let channel_client: Channel<Client, u32> = Channel::new(Box::new(s2), Box::new(r1));
                channel_client.close();
            })
        };

        {
            thread::spawn(|| {
                let channel_server: Channel<Protocol, u32> =
                    Channel::new(Box::new(s1), Box::new(r2));
                let cont = channel_server.do_action(Box::new(dummy), "hello".to_owned());
                cont.close();
            })
            .join()
            .unwrap();
        };
    }

    #[test]
    fn test_nested_tau() {
        type Protocol = Tau<Tau<Send<u32, Terminate>>>;
        type Reduced = <Protocol as Reducable>::Reduced;
        type Client = <Reduced as Action>::Dual;

        fn dummy(s: String) {
            assert_eq!(s, "hello")
        }

        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        {
            thread::spawn(|| {
                let num1 = 10_u32;
                let channel_client: Channel<Client, u32> = Channel::new(Box::new(s2), Box::new(r1));
                let (num, cont) = channel_client.recv();
                assert_eq!(num, num1);
                cont.close();
            })
        };

        {
            thread::spawn(|| {
                let num1 = 10_u32;
                let channel_server: Channel<Protocol, u32> =
                    Channel::new(Box::new(s1), Box::new(r2));
                let cont = channel_server.do_action(Box::new(dummy), "hello".to_owned());
                let cont = cont.do_action(Box::new(dummy), "hello".to_owned());
                let cont = cont.send(num1);
                cont.close()
            })
            .join()
            .unwrap();
        };
    }
}

mod extra_choice;
mod proto;
mod tcp;
