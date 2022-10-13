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

pub(crate) trait State: PartialEq + Ord + Default {}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct SessionBranch<T: State> {
    state: T,
    branch: Vec<T>,
}

impl<T: State> SessionBranch<T> {
    pub fn new(state: T) -> SessionBranch<T> {
        SessionBranch {
            state,
            branch: Vec::default(),
        }
    }

    pub(crate) fn get_iter(&self) -> Iter<T> {
        self.branch.iter()
    }

    pub(crate) fn get_iter_mut(&mut self) -> IterMut<T> {
        self.branch.iter_mut()
    }

    pub(crate) fn push_state(&mut self, state: T) -> bool {
        self.branch.push(state);
        self.branch.sort();
        true
    }

    pub(crate) fn contains(&self, state: &T) -> bool {
        self.branch.binary_search(state).is_ok()
    }
}

pub(crate) struct SessionType<'a, T: State> {
    protocol: Vec<&'a mut SessionBranch<T>>,
}

impl<'a, T: State> SessionType<'a, T> {
    pub(crate) fn new() -> SessionType<'a, T> {
        SessionType {
            protocol: Vec::default(),
        }
    }

    pub(crate) fn contains(&self, state: &T) -> bool {
        self.protocol
            .binary_search_by(|s| s.state.cmp(&state))
            .is_ok()
    }

    pub(crate) fn get_branch(&mut self, state: &T) -> Option<&mut SessionBranch<T>> {
        let index = match self.protocol.binary_search_by(|s| s.state.cmp(&state)) {
            Ok(i) => i,
            Err(_) => usize::MAX,
        };
        match self.protocol.get_mut(index) {
            Some(sb) => Some(sb),
            None => None,
        }
    }

    pub(crate) fn push_branch(&mut self, branch: &'a mut SessionBranch<T>) -> bool {
        if !self.contains(&branch.state) {
            self.protocol.push(branch);
            self.protocol.sort();
            return true;
        } else {
            return false;
        }
    }

    pub(crate) fn push_state(&mut self, onto: &T, state: T) -> bool {
        match self.get_branch(onto) {
            Some(b) => {
                b.push_state(state);
                return true;
            }
            None => return false,
        };
    }
}

pub(crate) struct SessionController<'a, T: State> {
    state: T,
    stype: SessionType<'a, T>,
}

#[cfg(test)]
mod tests {
    use crate::{tcp::TcpState, SessionBranch, SessionType};

    #[test]
    fn session_branch_push() {
        let mut sb = SessionBranch::new(TcpState::Closed);
        assert!(sb.push_state(TcpState::Listen));
        assert!(sb.push_state(TcpState::SynSent));
    }

    #[test]
    fn session_branch_contains() {
        let mut sb = SessionBranch::new(TcpState::Closed);
        assert!(sb.push_state(TcpState::Listen));
        assert!(sb.contains(&TcpState::Listen));
        assert!(sb.push_state(TcpState::SynSent));
        assert!(sb.contains(&TcpState::Listen));
        assert!(sb.contains(&TcpState::SynSent));
    }

    #[test]
    fn session_type_push_branch() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        sb_closed.push_state(TcpState::Listen);

        let mut st = SessionType::new();
        assert!(st.push_branch(&mut sb_closed));

        let mut sb_listen = SessionBranch::new(TcpState::Listen);
        sb_listen.push_state(TcpState::SynSent);

        assert!(st.push_branch(&mut sb_listen));
    }

    #[test]
    fn session_type_contains() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        sb_closed.push_state(TcpState::Listen);

        let mut st = SessionType::new();
        st.push_branch(&mut sb_closed);
        assert!(st.contains(&TcpState::Closed));

        let mut sb_listen = SessionBranch::new(TcpState::Listen);
        sb_listen.push_state(TcpState::SynSent);
        st.push_branch(&mut sb_listen);
        assert!(st.contains(&TcpState::Listen));
    }

    #[test]
    fn session_type_push_state() {
        let mut sb_closed = SessionBranch::new(TcpState::Closed);
        let mut st = SessionType::new();
        st.push_branch(&mut sb_closed);

        st.push_state(&TcpState::Closed, TcpState::Listen);

        assert!(sb_closed.contains(&TcpState::Listen));
    }
}

mod proto;
mod tcp;
