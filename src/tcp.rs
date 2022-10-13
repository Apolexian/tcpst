use crate::State;

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
    #[default] Closed,
}

impl State for TcpState {}
