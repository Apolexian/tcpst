use std::marker::PhantomData;

pub trait Action {
    fn new() -> Self
    where
        Self: Sized;
}

pub trait Message {}

enum BranchingsOfferToRoleA3Choices {
    Branch0,
    Branch1,
    Branch2,
}

enum BranchingsA3Choices<A, B, C> {
    One(A),
    Two(B),
    Three(C),
}

struct OfferToRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message,
    M1: Message,
    M2: Message,
    A0: Action,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(M0, M1, M2, A0, A1, A2)>,
}

impl<M0, M1, M2, A0, A1, A2> Action for OfferToRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message,
    M1: Message,
    M2: Message,
    A0: Action,
    A1: Action,
    A2: Action,
{
    fn new() -> Self {
        OfferToRoleA3Choices {
            phantom: PhantomData,
        }
    }
}

impl<M0, M1, M2, A0, A1, A2> OfferToRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message + 'static,
    M1: Message + 'static,
    M2: Message + 'static,
    A0: Action + 'static,
    A1: Action + 'static,
    A2: Action + 'static,
{
    pub fn offer<F>(
        self,
        emitter0: Box<dyn Fn() -> M0>,
        emitter1: Box<dyn Fn() -> M1>,
        emitter2: Box<dyn Fn() -> M2>,
    ) -> Box<dyn Fn(F) -> (Box<dyn Message>, Box<dyn Action>)>
    where
        F: Fn() -> BranchingsA3Choices<(M0, A0), (M1, A1), (M2, A2)>,
    {
        Box::new(move |picker| {
            let choice = picker();
            match choice {
                BranchingsA3Choices::One(_) => {
                    let message = emitter0();
                    return (Box::new(message), Box::new(A0::new()));
                }
                BranchingsA3Choices::Two(_) => {
                    let message = emitter1();
                    return (Box::new(message), Box::new(A1::new()));
                }
                BranchingsA3Choices::Three(_) => {
                    let message = emitter2();
                    return (Box::new(message), Box::new(A2::new()));
                }
            }
        })
    }
}
enum BranchingsSelectFromRoleA3Choices {
    Branch0,
    Branch1,
    Branch2,
}

struct SelectFromRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message,
    M1: Message,
    M2: Message,
    A0: Action,
    A1: Action,
    A2: Action,
{
    phantom: PhantomData<(M0, M1, M2, A0, A1, A2)>,
}

impl<M0, M1, M2, A0, A1, A2> Action for SelectFromRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message,
    M1: Message,
    M2: Message,
    A0: Action,
    A1: Action,
    A2: Action,
{
    fn new() -> Self {
        SelectFromRoleA3Choices {
            phantom: PhantomData,
        }
    }
}

impl<M0, M1, M2, A0, A1, A2> SelectFromRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message + 'static,
    M1: Message + 'static,
    M2: Message + 'static,
    A0: Action + 'static,
    A1: Action + 'static,
    A2: Action + 'static,
{
    pub fn select(
        self,
        emitter0: Box<dyn Fn(M0)>,
        emitter1: Box<dyn Fn(M1)>,
        emitter2: Box<dyn Fn(M2)>,
        message0: M0,
        message1: M1,
        message2: M2,
        picker: Box<dyn Fn() -> BranchingsSelectFromRoleA3Choices>,
    ) -> Box<dyn Action> {
        let choice = picker();
        match choice {
            BranchingsSelectFromRoleA3Choices::Branch0 => {
                emitter0(message0);
                return Box::new(A0::new());
            }
            BranchingsSelectFromRoleA3Choices::Branch1 => {
                emitter1(message1);
                return Box::new(A1::new());
            }
            BranchingsSelectFromRoleA3Choices::Branch2 => {
                emitter2(message2);
                return Box::new(A2::new());
            }
        }
    }
}

enum BranchingsSelectA3Choices<A, B, C> {
    One(A),
    Two(B),
    Three(C),
}

impl<M0, M1, M2, A0, A1, A2> SelectFromRoleA3Choices<M0, M1, M2, A0, A1, A2>
where
    M0: Message + 'static + Clone,
    M1: Message + 'static + Clone,
    M2: Message + 'static + Clone,
    A0: Action + 'static,
    A1: Action + 'static,
    A2: Action + 'static,
{
    pub fn offer<F>(
        self,
        emitter0: Box<dyn Fn(M0)>,
        emitter1: Box<dyn Fn(M1)>,
        emitter2: Box<dyn Fn(M2)>,
        message0: M0,
        message1: M1,
        message2: M2,
    ) -> Box<dyn Fn(F) -> Box<dyn Action>>
    where
        F: Fn() -> BranchingsSelectA3Choices<(M0, A0), (M1, A1), (M2, A2)>,
    {
        Box::new(move |picker| {
            let choice = picker();
            match choice {
                BranchingsSelectA3Choices::One(_) => {
                    emitter0(message0.clone());
                    return Box::new(A0::new());
                }
                BranchingsSelectA3Choices::Two(_) => {
                    emitter1(message1.clone());
                    return Box::new(A1::new());
                }
                BranchingsSelectA3Choices::Three(_) => {
                    emitter2(message2.clone());
                    return Box::new(A2::new());
                }
            }
        })
    }
}

struct End {}

impl Action for End {
    fn new() -> Self {
        End {}
    }
}
