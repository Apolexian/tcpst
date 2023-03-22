#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::net::Ipv4Addr;
use std::ops::Add;
use std::thread;

use crossbeam_channel::unbounded;
use mpstrust::crossbeam::{Close, Connected, CrossBeamRoleChannel, Open, TcbCreated};
use mpstrust::net_channel::{Ack, FinAck, NetChannel, Syn, SynAck};
use mpstrust::{Action, End, OfferOne, Role, SelectOne, SessionTypedChannel};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;

use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::tcp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use raw_socket::{Domain, Protocol, Type};

pub struct RoleServerSystem;
impl Role for RoleServerSystem {}

pub struct RoleServerUser;
impl Role for RoleServerUser {}

pub struct RoleServerClient;
impl Role for RoleServerClient {}

fn main() {
    let remote_addr = Ipv4Addr::new(127, 0, 0, 1);
    let local_addr = Ipv4Addr::new(127, 0, 0, 1);
    let source_port = 49155u16;

    // Silly trick to make the kernel not process TCP packets
    // this is used in combination with `iptables -A OUTPUT -p tcp --tcp-flags RST RST -j DROP`,
    // which drops any outgoing RST segments that the kernel tries to send.
    // This socket is never used again after but it means that the kernel will not try to process incoming segments.
    // https://stackoverflow.com/questions/31762305/prevent-kernel-from-processing-tcp-segments-bound-to-a-raw-socket
    let socket =
        raw_socket::RawSocket::new(Domain::ipv4(), Type::stream(), Some(Protocol::tcp())).unwrap();
    socket.bind(("127.0.0.1", source_port)).unwrap();

    // Create the underlying communication channel and the session typed NetChannel
    // net_channel models the communication between our TCP server and a remote client,
    // hence we bind it to corresponding roles.
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Tcp));
    let (tx, mut rx) = match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!(
            "An error occurred when creating the transport channel: {}",
            e
        ),
    };
    let iter = tcp_packet_iter(&mut rx);
    let mut net_channel =
        NetChannel::<RoleServerSystem, RoleServerClient>::new(iter, tx, remote_addr);

    // Create the underlying communication channel and the session typed CrossbeamChannel
    let (cbtx1, cbrx1) = unbounded();
    let (cbtx2, cbrx2) = unbounded();
    let mut system_user_channel =
        CrossBeamRoleChannel::<RoleServerSystem, RoleServerUser>::new(cbtx2, cbrx1);
    let mut user_system_channel =
        CrossBeamRoleChannel::<RoleServerUser, RoleServerSystem>::new(cbtx1, cbrx2);

    // Create and instantiate the session type of the local view of the TCP server.
    type ServerSystemSessionType = OfferOne<
        RoleServerUser,
        Open,
        SelectOne<
            RoleServerUser,
            TcbCreated,
            OfferOne<
                RoleServerClient,
                Syn,
                SelectOne<
                    RoleServerClient,
                    SynAck,
                    OfferOne<
                        RoleServerClient,
                        Ack,
                        SelectOne<
                            RoleServerUser,
                            Connected,
                            OfferOne<
                                RoleServerUser,
                                Close,
                                SelectOne<
                                    RoleServerClient,
                                    FinAck,
                                    SelectOne<RoleServerUser, Close, End>,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >;
    let st_system_server = ServerSystemSessionType::new();

    // Create and instantiate the session type of the local view of the TCP user.
    type ServerUserSessionType = SelectOne<
        RoleServerSystem,
        Open,
        OfferOne<
            RoleServerSystem,
            TcbCreated,
            OfferOne<
                RoleServerSystem,
                Connected,
                SelectOne<RoleServerSystem, Close, OfferOne<RoleServerSystem, Close, End>>,
            >,
        >,
    >;
    let st_sustem_user = ServerUserSessionType::new();

    thread::scope(|scope| {
        let thread_a = scope.spawn(|| {
            // Thread A simulates the kind of calls the userspace would send to the TCP system.
            // These are not actually implemented but it demonstrates the user of another
            // session typed channel on a different medium.
            // This also allows us to demonstrate the TCP system communicating with two sepparate participants.

            let cont = user_system_channel.select_one(st_sustem_user, Open {});
            let (_, cont) = user_system_channel.offer_one(cont);
            let (_, cont) = user_system_channel.offer_one(cont);
            let cont = user_system_channel.select_one(cont, Close {});
            let (_, cont) = user_system_channel.offer_one(cont);
            user_system_channel.close(cont);
        });
        let thread_b = scope.spawn(|| {
            // Thread B shows the communication from the point of the TCP system.
            // TCP system communicates with both the remote client and the local userspace.
            // The example only demonstrates establishing a handshake and then sending the closing packet.

            // Recieve the OPEN call from the user
            let (_, cont) = system_user_channel.offer_one(st_system_server);
            // Notify the user that the we are ready to accept an incoming connection
            let cont = system_user_channel.select_one(cont, TcbCreated {});

            // Recieve a SYN packet indicating the beginning of the opening handshake.
            let (syn_message, cont) = net_channel.offer_one(cont);

            // Construct a SYN-ACK packet and cast it to the appropriate message type.
            let packet = TcpPacket::new(&syn_message.packet).unwrap();
            let mut vec: Vec<u8> = vec![0; syn_message.packet.len()];
            let mut new_packet = MutableTcpPacket::new(&mut vec).unwrap();
            new_packet.set_flags(TcpFlags::ACK | TcpFlags::SYN);
            new_packet.set_sequence(1);
            new_packet.set_acknowledgement(packet.get_sequence().add(1));
            new_packet.set_source(packet.get_destination());
            new_packet.set_destination(packet.get_source());
            new_packet.set_window(packet.get_window());
            new_packet.set_data_offset(packet.get_data_offset());
            let checksum = ipv4_checksum(&new_packet.to_immutable(), &local_addr, &remote_addr);
            new_packet.set_checksum(checksum);
            let new_packet_slice = new_packet.packet();

            // Send the message along the channel, following our session type.
            let cont = net_channel.select_one(
                cont,
                SynAck {
                    packet: new_packet_slice.to_vec(),
                },
            );

            // Recieve a message of type ACK.
            let (ack_message, cont) = net_channel.offer_one(cont);
            let packet = TcpPacket::new(&ack_message.packet).unwrap();

            // Notify the user that the connection was established.
            let cont = system_user_channel.select_one(cont, Connected {});
            // Recieve the CLOSE call from the user
            let (_, cont) = system_user_channel.offer_one(cont);

            // For this example we will always just respond with a FIN-ACK and end.
            let mut vec: Vec<u8> = vec![0; packet.packet().len()];
            let mut new_packet = MutableTcpPacket::new(&mut vec).unwrap();
            new_packet.set_flags(TcpFlags::ACK | TcpFlags::FIN);
            new_packet.set_sequence(packet.get_acknowledgement());
            new_packet.set_acknowledgement(packet.get_sequence().add(1));
            new_packet.set_source(packet.get_destination());
            new_packet.set_destination(packet.get_source());
            new_packet.set_window(packet.get_window());
            new_packet.set_data_offset(packet.get_data_offset());
            let checksum = ipv4_checksum(&new_packet.to_immutable(), &local_addr, &remote_addr);
            new_packet.set_checksum(checksum);
            let new_packet_slice = new_packet.packet();

            // Send the FIN-ACK along the channel.
            let cont = net_channel.select_one(
                cont,
                FinAck {
                    packet: new_packet_slice.to_vec(),
                },
            );

            // Notify the user that the connection was closed.
            let cont = system_user_channel.select_one(cont, Close {});

            // End the session-typed communication, whichs drops the channel.
            net_channel.close(cont);
        });
        thread_a.join().unwrap();
        thread_b.join().unwrap();
    });
}
