# TCP-ST ICNP paper reviews

## Review #39A

Overall Merit: Weak Reject
Expertise: No familiarity

Paper summary

The paper applies multiparty session types to TCP specification to ensure automatic checking of correctness of implementations. The main results are a multiparty session type expression of the core functions of TCP, and a corresponding implementation in Rust. The implementation is tested for correctness to demonstrate the feasibility of the approach.
Strengths

    The idea of using formal languages such as multiparty session types to express protocol specifications has the promise of enabling automatic consistency checks and eliminating implementation errors.

    The example protocol of TCP is particularly relevant in terms of consistency checks due to the existence of many variants of the specification and their implementations.

    The authors have prototyped the idea and will open-source the code.

Weaknesses

    The paper is very hard to understand with its unusual terminologies and insufficient explanations.

    Given the unnatural nature of the proposed language (session types), it is questionable how useful the idea will be in practice. Requiring protocol developers to express their design in session types appears no easier than requiring them to directly code up the protocol in a given programming language. The proposed idea simply shifts the error-prone phase of work from the implementation of natural-language specification to the expression of protocol design ideas in a formal language. It is unclear why the latter is easier to do.

    Parts of the paper seem to have logic problems. See comments below for examples.

Comments for authors

    The terminologies used are counter-intuitive and not always explained, which makes it hard to verify correctness of the content.
    The text below (1) suggests that "label" stands for a time instant. If this is the case, then (1) has an obvious error that it requires a sends ping to b while b sends pong to a, both at instant l_1.
    Similarly, whether (1) is deadlock-free or not will depend on the order of the labels in time. It is not clear why the type (1) guarantees this property.

## Review #39B

Overall Merit: Weak Reject
Expertise: Some familiarity

Paper summary

The authors addressed these concerns by utilizing session types to demonstrate internal consistency, correctness, and conformance of implementations to the specifications. They present the first multiparty session type model of TCP and employ it to re-validate essential properties such as deadlock freedom and safety. Additionally, they leverage a novel encoding of session types in the Rust programming language to develop the inaugural session-typed TCP server implementation. This implementation serves as a compile-time check, ensuring adherence to the model derived from the specification and accurate execution of the TCP state machine.
Strengths

The paper explores the use of session types to enhance protocol specifications and implementations. They demonstrate that untyped protocols like TCP pose challenges when attempting to encode features such as SYN and ACK using generic type parameters. Instead, they propose using specific types to represent each valid combination of parameters.

The authors present the first session-typed model of the TCP state machine, showcasing the feasibility of using session types to model transport protocols and validate safety and liveness properties like deadlock freedom. They also develop an implementation of multiparty session types in Rust, providing an encoding method.

Furthermore, they utilize the encoding to create a session-typed Rust TCP server that enforces compliance with the TCP state machine. Notably, the server interoperates with existing TCP implementations, highlighting the novel aspect of being able to type-check low-level transport protocol implementations against a specification model.
Weaknesses

The paper's organization could be improved. While the motivation behind the problem is presented, it may not be sufficiently compelling. Similarly, the contributions made by the authors may not appear particularly significant. Moreover, there are concerns regarding the feasibility of implementing their work in practice. It would be beneficial for the authors to provide clearer explanations and address these concerns in their paper.

## Review #39C

Overall Merit: Reject
Expertise: Knowledgable

Paper summary

This paper uses multi-party session types (MPST) to verify the state machine of TCP and enforce its correct implementation.
Strengths

    It is an interesting topic.

Weaknesses

    Full TCP protocol stack should have been implemented with session types.

Comments for authors

    I believe that the paper may have some merit, but it looks like it is not finished. The whole TCP protocol stack should be implemented and evaluated using session types. We should understand if TCP with and without session types achieve similar performance.

    It is difficult to follow the code in Fig. 5. The authors may consider simplifying it.

    Language of the paper should be improved.

    It is not clear from the abstract what the paper would like to do.
