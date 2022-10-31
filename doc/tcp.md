# TCP

 Notes on features of TCP and their modelling

## Connection establishment

Notes: 

* A TCP implementation MUST support simultaneous open attempts (MUST-10).


https://datatracker.ietf.org/doc/html/rfc9293#section-3.5

| Feature                                                      | Model                            |
| ------------------------------------------------------------ | -------------------------------- |
| Basic Three-Way Handshake <br>for Connection Synchronization | ST-Primitives + Value Dependence |
| Simultaneous open attempts                                   | ST-Primitives + Value Dependence |
| Recovery from Old Duplicate SYN                              |                                  |
| Half-Open Connections                                        |                                  |
| Old Duplicate SYN Initiates a Reset on Two Passive Sockets   |                                  |
| Active Side Causes Half-Open Connection Discovery            |                                  |

## Modeling tags

| Tag              | Note                                                                                                                                                                                                                                                                                               |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ST-Primitives    | Primitives session type operations. <br> I.e. `&, ⊕, ?, !, end`.                                                                                                                                                                                                                                   |
| Value Dependence | The protocol depends on a specific value, e.g. the check for the incrementing of sequence numbers depends on the actual value. <br> This means we can either parse a packet, and do this check outside of the formal state machine or the formalism needs to be able to do value dependent typing. |
|                  |                                                                                                                                                                                                                                                                                                    |