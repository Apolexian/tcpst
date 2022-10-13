use std::default;

use crate::{Message, State};

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

#[derive(Ord, PartialEq, PartialOrd, Eq, Default)]
pub(crate) enum TcpMessage {
    #[default]
    Segment,
    Abort,
    Open,
    ServerSyn,
    ServerAck,
    Recieve,
    Close,
    ServerRst,
    SynRecieved,
    Send,
}

impl Message for TcpMessage {}

impl State for TcpState {}
