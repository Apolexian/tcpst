use std::marker::PhantomData;

use crossbeam_channel::{Receiver, Sender};

use crate::Role;

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

