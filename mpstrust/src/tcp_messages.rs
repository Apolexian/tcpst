use crate::Message;

pub struct Ack {
}
impl Message for Ack {}
impl AckMessage for Ack{}

pub trait SynMessage: Message {}
pub trait AckMessage: Message {}

pub struct Syn {
}
impl Message for Syn {}
impl SynMessage for Syn{}