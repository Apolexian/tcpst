use crate::Message;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TcpState {
    Listen,
    SynSent,
    SynRecieved,
    Established,
    FinWaitOne,
    FinWaitTwo,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    #[default]
    Closed,
}

pub trait TcpMessage: Message {}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Segment {}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Abort {}

impl Message for Segment {}
impl Message for Abort {}
impl TcpMessage for Segment {}
impl TcpMessage for Abort {}
