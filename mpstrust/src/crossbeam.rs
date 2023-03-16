use std::marker::PhantomData;

use crossbeam_channel::{Receiver, Sender};

use crate::{
    Message, Role,
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
}

pub trait CBMessage: Message {
    fn from_slice(_slice: Vec<u8>) -> Self;
    fn to_slice(&self) -> Vec<u8>;
}

pub struct CBAck {}
impl Message for CBAck {}
impl CBMessage for CBAck {
    fn from_slice(_slice: Vec<u8>) -> Self {
        CBAck {}
    }

    fn to_slice(&self) -> Vec<u8> {
        vec![]
    }
}

pub struct CBSyn {}
impl Message for CBSyn {}
impl CBMessage for CBSyn {
    fn from_slice(_slice: Vec<u8>) -> Self {
        CBSyn {}
    }

    fn to_slice(&self) -> Vec<u8> {
        vec![]
    }
}
