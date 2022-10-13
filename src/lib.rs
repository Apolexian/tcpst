#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
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

pub trait Session {
    fn get_dual(&self) -> Self;
}

pub struct Send<M: Message, S: Session> {
    message: M,
    progress: S,
}

impl<M: Message, S: Session> Session for Send<M, S> {
    fn get_dual(&self) -> Self {
        todo!()
    }
}

pub struct Recv<M: Message, S: Session> {
    message: M,
    progress: S,
    dual: S,
}

pub struct End {}

impl Session for End {
    fn get_dual(&self) -> Self {
        todo!()
    }
}

impl<M: Message, S: Session> Session for Recv<M, S> {
    fn get_dual(&self) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tcp::{Abort, Segment},
        End, Recv, Send,
    };

    #[test]
    fn whatever() {
        type TcpServer = Recv<Segment, Send<Abort, End>>;
    }
}

mod proto;
mod tcp;
