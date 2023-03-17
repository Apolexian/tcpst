use crate::{Branch, Role, SessionTypedChannel};
use std::marker::PhantomData;
use tun_tap::Iface;

pub struct NetChannel<R1, R2>
where
    R1: Role,
    R2: Role,
{
    phantom: PhantomData<(R1, R2)>,
    nic: Iface,
}

impl<R1, R2> SessionTypedChannel<R1, R2> for NetChannel<R1, R2>
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
        let mut buf = [0u8; 1500];
        let read = self.nic.recv(&mut buf).unwrap();
        let message = M::from_net_representation(buf[..read].to_vec());
        (message, A::new())
    }

    fn select_one<M, A>(&mut self, _o: crate::SelectOne<R2, M, A>, message: M) -> A
    where
        M: crate::Message,
        A: crate::Action,
        R1: Role,
        R2: Role,
    {
        let packet = message.to_net_representation();
        self.nic.send(&packet).unwrap();
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
        let mut buf = [0u8; 1500];
        match choice {
            true => {
                let read = self.nic.recv(&mut buf).unwrap();
                let message = M1::from_net_representation(buf[..read].to_vec());
                Branch::Left((message, A1::new()))
            }
            false => {
                let read = self.nic.recv(&mut buf).unwrap();
                let message = M2::from_net_representation(buf[..read].to_vec());
                Branch::Right((message, A2::new()))
            }
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
        let packet = message.to_net_representation();
        self.nic.send(&packet).unwrap();
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
        let packet = message.to_net_representation();
        self.nic.send(&packet).unwrap();
        A2::new()
    }

    fn close(self, _end: crate::End) {
        drop(self);
    }
}

impl<R1, R2> NetChannel<R1, R2>
where
    R1: Role,
    R2: Role,
{
    pub fn new(nic: Iface) -> Self {
        NetChannel {
            phantom: PhantomData::default(),
            nic,
        }
    }
}
