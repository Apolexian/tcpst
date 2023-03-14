use std::marker::PhantomData;

use crossbeam_channel::{Receiver, Sender};

use crate::{
    tcp_messages::{Ack, Syn},
    Role,
};

#[derive(Clone)]
pub struct CrossBeamRoleChannel<R1, R2>
where
    R1: Role,
    R2: Role,
{
    pub send: Sender<Vec<u8>>,
    pub recv: Receiver<Vec<u8>>,
    pub phantom: PhantomData<(R1, R2)>,
}

impl<R1, R2> CrossBeamRoleChannel<R1, R2>
where
    R1: Role,
    R2: Role,
{
    pub fn new(send: Sender<Vec<u8>>, recv: Receiver<Vec<u8>>) -> Self {
        CrossBeamRoleChannel {
            send,
            recv,
            phantom: PhantomData::default(),
        }
    }

    pub fn to_ack_message(_slice: Vec<u8>) -> Ack {
        Ack {}
    }

    pub fn from_ack_message(_ack: Ack) -> Vec<u8> {
        vec![]
    }

    pub fn to_syn_message(_slice: Vec<u8>) -> Syn {
        Syn {}
    }

    pub fn from_syn_message(_ack: Syn) -> Vec<u8> {
        vec![]
    }
}
