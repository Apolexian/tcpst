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

pub(crate) enum TcpMessage {
    Segment(Segment),
    Abort(Abort),
}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Segment {}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Abort {}
