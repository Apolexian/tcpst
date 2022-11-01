# TCP

 Notes on features of TCP and their modelling.

## General considerations

The modelling considerations are often described/noted in terms of session types. 
Despite this choice the modeling techniques could be viewed in the form of any formalism that can express the required actions.

There are also some general features and cases that apply to almost every part of the communication.
Without these the communication will simply not work, hence these are extracted here:


| Feature/Branch Case       | Model               |
| ------------------------- | ------------------- |
| Unknown number of actions | Unbounded recursion |

Notes:

* `Unknown number of actions`. We cannot know ahead of time how many times a server will need to recieve a packet, or how many times a client will need to send one. To model this we need some recursive environment that may be unbounded.

## Sequence Numbers

https://datatracker.ietf.org/doc/html/rfc9293#section-3.4

| Feature/Branch Case                               | Model                                    |
| ------------------------------------------------- | ---------------------------------------- |
| Clock                                             | Outside Consideration                    |
| Keeping quiet when <br>assigning sequence numbers | Timed + Outside Consideration            |
| TCP Quiet Time Concept                            | ST-Primitives + Value Dependence + Timed |

  Notes:

  * `TCP Quiet Time Concept`. Need to keep track of the last sequence number, so `Value Dependence`. Hosts that for any reason lose knowledge of the last sequence numbers transmitted on each active (i.e., not closed) connection shall delay emitting any TCP segments for at least the agreed MSL in the internet system that the host is a part of, so `Timed`.
  * `Keeping quiet when assigning sequence numbers` - the TCP endpoint must keep quiet for an `MSL` before assigning any sequence numbers upon starting up or recovering from a situation where memory of sequence numbers in use was lost. From my understading, assigning a sequence number means generating (instantiating) it, so this is part of an outside algorithm. However, waiting for an `MSL` effects the communication.

## Connection establishment

https://datatracker.ietf.org/doc/html/rfc9293#section-3.5

| Feature/Branch Case                                          | Model                            |
| ------------------------------------------------------------ | -------------------------------- |
| Basic Three-Way Handshake <br>for Connection Synchronization | ST-Primitives + Value Dependence |
| Simultaneous open attempts                                   | ST-Primitives + Value Dependence |
| Recovery from Old Duplicate SYN                              | ST-Primitives + Value Dependence |
| Half-Open Connections                                        | ST-Primitives + Value Dependence |
| Old Duplicate SYN Initiates a Reset on Two Passive Sockets   | ST-Primitives + Value Dependence |
| Active Side Causes Half-Open Connection Discovery            | ST-Primitives + Value Dependence |

Notes:

* Abort states are explicit so can be modelled as `end`
* All have `Value Dependence` as `SEQ` and `ACK` fields are dependant
* A TCP implementation MUST support simultaneous open attempts (MUST-10)

## Reset Generation

https://datatracker.ietf.org/doc/html/rfc9293#section-3.5.2

| Feature/Branch Case                             | Model                            |
| ----------------------------------------------- | -------------------------------- |
| Connection does not exist                       | ST-Primitives + Value Dependence |
| Connection is in any <br>non-synchronized state | ST-Primitives + Value Depenednce |
| Connection is in a synchronized state           | ST-Primitives + Value Dependence |
| IP Security Compartment and Precedence          | Value Dependence                 |

Notes:

* All have `Value Dependence` due. In the first two cases the `SEQ` of the reset depends on the `ACK` field if `ACK` is set. On top of this, if the `ACK` field in not set then the the reset has sequence number zero and the ACK field is set to the sum of the sequence number and segment length of the incoming segment
* On `IP Security Compartment and Precedence` (https://datatracker.ietf.org/doc/html/rfc9293#appendix-A.1) - the Diffserv field value used in each direction is a part of the interface between TCP and the network layer, and values in use can be indicated both ways between TCP and the application.

## Reset Processing

https://datatracker.ietf.org/doc/html/rfc9293#section-3.5.3

| Feature/Branch Case      | Model                            |
| ------------------------ | -------------------------------- |
| Reset segment validation | ST-Primitives + Value Dependence |

Notes:

* `Value Dependence` due to `SEQ` field (used for validation) needing to be in the window.
* TCP implementations SHOULD allow a received RST segment to include data (SHLD-2).  It has been suggested that a RST segment could contain diagnostic data that explains the cause of the RST.  No standard has yet been established for such data.

## Closing a Connection

| Feature/Branch Case                                                 | Model                                    |
| ------------------------------------------------------------------- | ---------------------------------------- |
| Recieve until told that <br>remote has closed                       | ST-Primitives + Failure                  |
| Signal that remote peer has closed                                  | ST-Primitives                            |
| Reliably deliver all buffers sent <br> before connection was closed | ST-Primitives                            |
| Initiate close                                                      | ST-Primitives                            |
| Remote sends FIN control signal                                     | ST-Primitives + Failure                  |
| Simultaneous close                                                  | ST-Primitives                            |
| Half-duplex close sequence                                          | ST-Primitives                            |
| Active close time lingering                                         | ST-Primitives + Timed + Value Dependence |
| Timestamp Options                                                   | ST-Primitives + Value Dependence         |

Notes:

* `Timed` for `Active close time lingering`: When a connection is closed actively, it MUST linger in the `TIME-WAIT` state for a time `2xMSL` (Maximum Segment Lifetime) (MUST-13). However, it MAY accept a new SYN from the remote TCP endpoint to reopen the connection directly from `TIME-WAIT` state (MAY-2). This happens if it assigns its initial sequence number for the new connection to be larger than the largest sequence number it used on the previous connection incarnation, and returns to `TIME-WAIT` state if the SYN turns out to be an old duplicate.
* `Timestamp Options` (https://datatracker.ietf.org/doc/html/rfc6191). `Value Dependence` as new `Timestamp` field would cause dependencies and dictate state transitions. This algorithm for reducing TIME-WAIT is a Best Current Practice that SHOULD be implemented since Timestamp Options are commonly used, and using them to reduce TIME-WAIT provides benefits for busy Internet servers (SHLD-4).

## Modeling tags

| Tag                   | Note                                                                                                                                                                                                                                                                                                                                                                                               |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ST-Primitives         | Primitives session type operations. <br> I.e. `&, ⊕, ?, !, end`.                                                                                                                                                                                                                                                                                                                                   |
| Value Dependence      | The protocol depends on a specific value, e.g. the check for the incrementing of sequence numbers depends on the actual value. <br> This means we can either parse a packet, and do this check outside of the formal state machine or the formalism needs to be able to do value dependent typing. In the case where we lift this to an outside algorithm this becomes an `Outside Consideration`. |
| Failure               | Indicates that a an action was performed but failed. The modelling of this depends on what failure entails. This could be a timeout or some sort of explicit error from I/O.                                                                                                                                                                                                                       |
| Timed                 | Explicit time bounds such as ones required by the `TIME-WAIT` state. This can be lifted from the state machine and handled by some abstract clock. In this case this would become an `Outside Consideration` and the state machine would only need to be informed when it can perform an action.                                                                                                   |
| Unbounded recursion   | Unbounded recursion is when we can't predict the number of repetitions for a recursive function/action.                                                                                                                                                                                                                                                                                            |
| Outside Consideration | A feature that relies on an algorithm that is outside the consideration of the actual communication/state machine. For example, the algorithm for generating initial sequence numbers is precisely defined and required, however, this is outside the bounds of the communication protocol.                                                                                                        |