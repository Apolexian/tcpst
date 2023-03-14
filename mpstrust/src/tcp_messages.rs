use crate::Message;

pub struct Ack {}
impl Message for Ack {
    fn from_slice(_slice: Vec<u8>) -> Self {
        Ack {}
    }

    fn to_slice(&self) -> Vec<u8> {
        vec![]
    }
}
impl AckMessage for Ack {}

pub trait SynMessage: Message {}
pub trait AckMessage: Message {}

pub struct Syn {}
impl Message for Syn {
    fn from_slice(_slice: Vec<u8>) -> Self {
        Syn {}
    }

    fn to_slice(&self) -> Vec<u8> {
        vec![]
    }
}
impl SynMessage for Syn {}
