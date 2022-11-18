use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use crate::{Action, Channel, Offer};

pub enum Three<F, S, T> {
    First(F),
    Second(S),
    Third(T),
}

pub enum ThreeChoice {
    One,
    Two,
    Three,
}

pub enum Four<F, S, T, Fo> {
    First(F),
    Second(S),
    Third(T),
    Fourth(Fo),
}

pub enum FourChoice {
    One,
    Two,
    Three,
    Four,
}

pub struct OfferThree<M, A, O, S>
where
    A: Action,
    O: Action,
    S: Action,
{
    phantom: PhantomData<(M, A, O, S)>,
}

impl<M, A, O, S> Action for OfferThree<M, A, O, S>
where
    A: Action,
    O: Action,
    S: Action,
{
    type Dual = ChooseThree<M, A::Dual, O::Dual, S::Dual>;
}

impl<A: Action, O: Action, M, T, S: Action> Channel<OfferThree<M, A, O, S>, T> {
    #[must_use]
    pub fn offer(
        self,
        choice: Box<dyn Fn() -> ThreeChoice>,
    ) -> Three<Channel<A, T>, Channel<O, T>, Channel<S, T>> {
        let choice = choice();
        unsafe {
            let pin = ManuallyDrop::new(self);
            let sender = ptr::read(&(pin).sender);
            let receiver = ptr::read(&(pin).receiver);
            match choice {
                ThreeChoice::One => Three::First(Channel::new(sender, receiver)),
                ThreeChoice::Two => Three::Second(Channel::new(sender, receiver)),
                ThreeChoice::Three => Three::Third(Channel::new(sender, receiver)),
            }
        }
    }
}

pub struct OfferFour<M, A, O, S, R>
where
    A: Action,
    O: Action,
    S: Action,
    R: Action,
{
    phantom: PhantomData<(M, A, O, S, R)>,
}

impl<M, A, O, S, R> Action for OfferFour<M, A, O, S, R>
where
    A: Action,
    O: Action,
    S: Action,
    R: Action,
{
    type Dual = ChooseFour<M, A::Dual, O::Dual, S::Dual, R::Dual>;
}

impl<A: Action, O: Action, M, T, S: Action, R: Action> Channel<OfferFour<M, A, O, S, R>, T> {
    #[must_use]
    pub fn offer(
        self,
        choice: Box<dyn Fn() -> FourChoice>,
    ) -> Four<Channel<A, T>, Channel<O, T>, Channel<S, T>, Channel<R, T>> {
        let choice = choice();
        unsafe {
            let pin = ManuallyDrop::new(self);
            let sender = ptr::read(&(pin).sender);
            let receiver = ptr::read(&(pin).receiver);
            match choice {
                FourChoice::One => Four::First(Channel::new(sender, receiver)),
                FourChoice::Two => Four::Second(Channel::new(sender, receiver)),
                FourChoice::Three => Four::Third(Channel::new(sender, receiver)),
                FourChoice::Four => Four::Fourth(Channel::new(sender, receiver)),
            }
        }
    }
}

pub struct ChooseThree<M, A, O, S>
where
    A: Action,
    O: Action,
    S: Action,
{
    phantom: PhantomData<(M, A, O, S)>,
}

impl<M, A, O, S> Action for ChooseThree<M, A, O, S>
where
    A: Action,
    O: Action,
    S: Action,
{
    type Dual = OfferThree<M, A::Dual, O::Dual, S::Dual>;
}

impl<A: Action, O: Action, M, T, S: Action> Channel<ChooseThree<M, A, O, S>, T> {
    pub fn choose_first(self) -> Channel<A, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_second(self) -> Channel<O, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_third(self) -> Channel<S, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }
}

pub struct ChooseFour<M, A, O, S, R>
where
    A: Action,
    O: Action,
    S: Action,
    R: Action,
{
    phantom: PhantomData<(M, A, O, S, R)>,
}

impl<M, A, O, S, R> Action for ChooseFour<M, A, O, S, R>
where
    A: Action,
    O: Action,
    S: Action,
    R: Action,
{
    type Dual = OfferFour<M, A::Dual, O::Dual, S::Dual, R::Dual>;
}

impl<A: Action, O: Action, M, T, S: Action, R: Action> Channel<ChooseFour<M, A, O, S, R>, T> {
    pub fn choose_first(self) -> Channel<A, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_second(self) -> Channel<O, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_third(self) -> Channel<S, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }

    pub fn choose_fourth(self) -> Channel<R, T> {
        unsafe {
            let pin = ManuallyDrop::new(self);
            Channel::new(ptr::read(&(pin).sender), ptr::read(&(pin).receiver))
        }
    }
}
