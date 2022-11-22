use mpstthree::binary::struct_trait::{end::End, recv::Recv, send::Send};
use mpstthree::meshedchannels::MeshedChannels;
use mpstthree::role::a::RoleA;
use mpstthree::role::b::RoleB;
use mpstthree::role::c::RoleC;
use mpstthree::role::end::RoleEnd;

type AtoB = Send<u8, End>;
type AtoC = Recv<u8, End>;

type BtoA = Recv<u8, End>;
type BtoC = Send<u8, End>;

type CtoA = Send<u8, End>;
type CtoB = Recv<u8, End>;

type StackA = RoleB<RoleC<RoleEnd>>;
type StackB = RoleA<RoleC<RoleEnd>>;
type StackC = RoleA<RoleB<RoleEnd>>;

type NameA = RoleA<RoleEnd>;
type NameB = RoleB<RoleEnd>;
type NameC = RoleC<RoleEnd>;

type EndpointA = MeshedChannels<AtoB, AtoC, StackA, NameA>;
type EndpointB = MeshedChannels<BtoA, BtoC, StackB, NameB>;
type EndpointC = MeshedChannels<CtoA, CtoB, StackC, NameC>;

#[cfg(test)]
mod tests {
    use std::error::Error;

    use mpstthree::functionmpst::fork::fork_mpst;

    use super::{EndpointA, EndpointB, EndpointC};

    fn endpoint_a(s: EndpointA) -> Result<(), Box<dyn Error>> {
        let s = s.send(1);
        let (x, s) = s.recv()?;
        assert_eq!(x, 3);
        s.close()
    }

    fn endpoint_b(s: EndpointB) -> Result<(), Box<dyn Error>> {
        let (x, s) = s.recv()?;
        let s = s.send(2);
        assert_eq!(x, 1);
        s.close()
    }

    fn endpoint_c(s: EndpointC) -> Result<(), Box<dyn Error>> {
        let s = s.send(3);
        let (x, s) = s.recv()?;
        assert_eq!(x, 2);
        s.close()
    }

    #[test]
    fn simple_example_works() {
        let (thread_a, thread_b, thread_c) = fork_mpst(endpoint_a, endpoint_b, endpoint_c);

        thread_a.join().unwrap();
        thread_b.join().unwrap();
        thread_c.join().unwrap();
    }
}
