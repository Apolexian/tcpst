#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use std::slice::{Iter, IterMut};

/// TODO:
/// 1) Need some sort of way to represent a session type, i.e. encode the TCP state machine in types
///
/// 2) Need some way to decide if the path we are taking is acceptable by the session type
///
/// 3) Need to decide what happens if the path is not acceptable
///
/// 4) Need to decide how/what the communication between the client and server consists of
///

pub trait State: PartialEq + Ord + Default {}

pub trait Message: PartialEq + Ord + Default {}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageTransition<T: State, M: Message> {
    message: M,
    new_state: T,
}

impl<T: State, M: Message> MessageTransition<T, M> {
    pub fn new(message: M, resulting_state: T) -> MessageTransition<T, M> {
        MessageTransition {
            message,
            new_state: resulting_state,
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct SessionBranch<T: State, M: Message> {
    state: T,
    branch: Vec<MessageTransition<T, M>>,
}

impl<T: State, M: Message> SessionBranch<T, M> {
    pub fn new(state: T) -> SessionBranch<T, M> {
        SessionBranch {
            state,
            branch: Vec::default(),
        }
    }

    pub fn get_iter(&self) -> Iter<MessageTransition<T, M>> {
        self.branch.iter()
    }

    pub fn get_iter_mut(&mut self) -> IterMut<MessageTransition<T, M>> {
        self.branch.iter_mut()
    }

    pub fn push_state(&mut self, message_transition: MessageTransition<T, M>) -> bool {
        self.branch.push(message_transition);
        self.branch.sort();
        true
    }

    pub(crate) fn contains(&self, message_transition: &MessageTransition<T, M>) -> bool {
        self.branch.binary_search(message_transition).is_ok()
    }
}

pub(crate) struct SessionType<'a, T: State, M: Message> {
    protocol: Vec<&'a mut SessionBranch<T, M>>,
}

impl<'a, T: State, M: Message> SessionType<'a, T, M> {
    pub fn new() -> SessionType<'a, T, M> {
        SessionType {
            protocol: Vec::default(),
        }
    }

    pub fn contains(&self, state: &T) -> bool {
        self.protocol
            .binary_search_by(|s| s.state.cmp(&state))
            .is_ok()
    }

    pub fn get_branch(&mut self, state: &T) -> Option<&mut SessionBranch<T, M>> {
        let index = match self.protocol.binary_search_by(|s| s.state.cmp(&state)) {
            Ok(i) => i,
            Err(_) => usize::MAX,
        };
        match self.protocol.get_mut(index) {
            Some(sb) => Some(sb),
            None => None,
        }
    }

    pub fn push_branch(&mut self, branch: &'a mut SessionBranch<T, M>) -> bool {
        if !self.contains(&branch.state) {
            self.protocol.push(branch);
            self.protocol.sort();
            return true;
        } else {
            return false;
        }
    }

    pub fn push_state(&mut self, onto: &T, message_transition: MessageTransition<T, M>) -> bool {
        match self.get_branch(onto) {
            Some(b) => {
                b.push_state(message_transition);
                return true;
            }
            None => return false,
        };
    }
}

pub(crate) struct SessionController<'a, T: State, M: Message> {
    state: T,
    stype: SessionType<'a, T, M>,
}

#[cfg(test)]
mod tests {
    use crate::{
        tcp::{TcpMessage, TcpState},
        MessageTransition, SessionBranch, SessionType,
    };

    #[test]
    fn session_branch_push() {
        let mut sb = SessionBranch::new(TcpState::Closed);
        let msg_closed_open_listen = MessageTransition::new(TcpMessage::Open, TcpState::Listen);
        let msg_closed_syn_sent = MessageTransition::new(TcpMessage::ServerSyn, TcpState::SynSent);
        assert!(sb.push_state(msg_closed_open_listen));
        assert!(sb.push_state(msg_closed_syn_sent));
    }

    #[test]
    fn session_branch_contains() {
        let mut sb = SessionBranch::new(TcpState::Closed);
        let msg_closed_open_listen = MessageTransition::new(TcpMessage::Open, TcpState::Listen);
        assert!(sb.push_state(msg_closed_open_listen));
        assert!(sb.contains(&MessageTransition::new(TcpMessage::Open, TcpState::Listen)));
        let msg_closed_syn_sent = MessageTransition::new(TcpMessage::ServerSyn, TcpState::SynSent);
        assert!(sb.push_state(msg_closed_syn_sent));
        assert!(sb.contains(&MessageTransition::new(TcpMessage::Open, TcpState::Listen)));
        assert!(sb.contains(&MessageTransition::new(
            TcpMessage::ServerSyn,
            TcpState::SynSent
        )));
    }

    #[test]
    fn session_type_push_branch() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        let msg_closed_open_listen = MessageTransition::new(TcpMessage::Open, TcpState::Listen);
        sb_closed.push_state(msg_closed_open_listen);

        let mut st = SessionType::new();
        assert!(st.push_branch(&mut sb_closed));

        let mut sb_listen = SessionBranch::new(TcpState::Listen);
        let msg_closed_syn_sent = MessageTransition::new(TcpMessage::ServerSyn, TcpState::SynSent);
        sb_listen.push_state(msg_closed_syn_sent);

        assert!(st.push_branch(&mut sb_listen));
    }

    #[test]
    fn session_type_contains() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        let msg_closed_open_listen = MessageTransition::new(TcpMessage::Open, TcpState::Listen);
        sb_closed.push_state(msg_closed_open_listen);

        let mut st = SessionType::new();
        st.push_branch(&mut sb_closed);
        assert!(st.contains(&TcpState::Closed));

        let mut sb_listen = SessionBranch::new(TcpState::Listen);
        let msg_closed_syn_sent = MessageTransition::new(TcpMessage::ServerSyn, TcpState::SynSent);
        sb_listen.push_state(msg_closed_syn_sent);
        st.push_branch(&mut sb_listen);
        assert!(st.contains(&TcpState::Listen));
    }

    #[test]
    fn session_type_push_state() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        let msg_closed_open_listen = MessageTransition::new(TcpMessage::Open, TcpState::Listen);
        let mut st = SessionType::new();
        st.push_branch(&mut sb_closed);

        st.push_state(&TcpState::Closed, msg_closed_open_listen);

        assert!(sb_closed.contains(&MessageTransition::new(TcpMessage::Open, TcpState::Listen)));
    }
}

mod proto;
mod tcp;
