/**
 * Copyright 2023, Ivan Nikitin.
 * This file is part of TCP-ST.
 *
 * TCP-ST is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
 *
 * TCP-ST is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with TCP-ST.
 * If not, see <https://www.gnu.org/licenses/>.
 *
 */
use std::marker::PhantomData;

use crossbeam_channel::{Receiver, Sender};

use crate::{Branch, Message, Role, SessionTypedChannel};

/// [CrossBeamRoleChannel] is a session-typed communication channel that uses crossbeam channels under the hood.
/// [CrossBeamRoleChannel] behaves as any other session-typed channels and implements [SessionTypedChannel].
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

impl<R1, R2> SessionTypedChannel<R1, R2> for CrossBeamRoleChannel<R1, R2>
where
    R1: Role,
    R2: Role,
{
    fn offer_one<M, A>(&mut self, _o: crate::OfferOne<R2, M, A>) -> (M, A)
    where
        M: crate::Message + 'static,
        A: crate::Action + 'static,
        R1: Role,
        R2: Role,
    {
        (
            M::from_net_representation(self.recv.recv().unwrap()),
            A::new(),
        )
    }

    fn select_one<M, A>(&mut self, _o: crate::SelectOne<R2, M, A>, message: M) -> A
    where
        M: crate::Message,
        A: crate::Action,
        R1: Role,
        R2: Role,
    {
        self.send.send(message.to_net_representation()).unwrap();
        A::new()
    }

    fn offer_two<M1, M2, A1, A2>(
        &mut self,
        _o: crate::OfferTwo<R2, M1, M2, A1, A2>,
        picker: Box<dyn Fn() -> bool>,
    ) -> crate::Branch<(M1, A1), (M2, A2)>
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        let choice = picker();
        match choice {
            true => Branch::Left((
                M1::from_net_representation(self.recv.recv().unwrap()),
                A1::new(),
            )),
            false => Branch::Right((
                M2::from_net_representation(self.recv.recv().unwrap()),
                A2::new(),
            )),
        }
    }

    fn select_left<M1, M2, A1, A2>(
        &mut self,
        _o: crate::SelectTwo<R2, M1, M2, A1, A2>,
        message: M1,
    ) -> A1
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        self.send.send(message.to_net_representation()).unwrap();
        A1::new()
    }

    fn select_right<M1, M2, A1, A2>(
        &mut self,
        _o: crate::SelectTwo<R2, M1, M2, A1, A2>,
        message: M2,
    ) -> A2
    where
        R1: Role,
        R2: Role,
        M1: crate::Message + 'static,
        M2: crate::Message + 'static,
        A1: crate::Action,
        A2: crate::Action,
    {
        self.send.send(message.to_net_representation()).unwrap();
        A2::new()
    }

    fn close(self, _end: crate::End) {
        drop(self);
    }
}

pub struct Open {}

impl Message for Open {
    fn to_net_representation(self) -> Vec<u8> {
        vec![]
    }

    fn from_net_representation(_packet: Vec<u8>) -> Self {
        Open {}
    }
}

pub struct TcbCreated {}

impl Message for TcbCreated {
    fn to_net_representation(self) -> Vec<u8> {
        vec![]
    }

    fn from_net_representation(_packet: Vec<u8>) -> Self {
        TcbCreated {}
    }
}

pub struct Close {}

impl Message for Close {
    fn to_net_representation(self) -> Vec<u8> {
        vec![]
    }

    fn from_net_representation(_packet: Vec<u8>) -> Self {
        Close {}
    }
}

pub struct Connected {}

impl Message for Connected {
    fn to_net_representation(self) -> Vec<u8> {
        vec![]
    }

    fn from_net_representation(_packet: Vec<u8>) -> Self {
        Connected {}
    }
}
