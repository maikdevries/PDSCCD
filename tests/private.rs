use pcd::private::{
    core::{Graph, Location, Node, Participant},
    crypto::STTP,
    protocol::Protocol,
};

#[test]
fn success() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            6,
            &sttp,
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        Participant::new(
            "B",
            6,
            &sttp,
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        Participant::new(
            "C",
            6,
            &sttp,
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn no_external_nodes() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            6,
            &sttp,
            Graph::new([
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, []),
            ]),
        ),
        Participant::new(
            "B",
            6,
            &sttp,
            Graph::new([
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, []),
            ]),
        ),
        Participant::new(
            "C",
            6,
            &sttp,
            Graph::new([
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn no_external_incoming() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            6,
            &sttp,
            Graph::new([
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        Participant::new(
            "B",
            6,
            &sttp,
            Graph::new([
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        Participant::new(
            "C",
            6,
            &sttp,
            Graph::new([
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn no_external_outgoing() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            6,
            &sttp,
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, []),
            ]),
        ),
        Participant::new(
            "B",
            6,
            &sttp,
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, []),
            ]),
        ),
        Participant::new(
            "C",
            6,
            &sttp,
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn component_size_exceeded() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            5,
            &sttp,
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        Participant::new(
            "B",
            5,
            &sttp,
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        Participant::new(
            "C",
            5,
            &sttp,
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn participant_specific_component_size() {
    let sttp = STTP::new();

    let participants = [
        Participant::new(
            "A",
            5,
            &sttp,
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        Participant::new(
            "B",
            6,
            &sttp,
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        Participant::new(
            "C",
            7,
            &sttp,
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a["A"].is_empty() && !a["B"].is_empty() && !a["C"].is_empty());

    assert_eq!(a, b);
    assert_eq!(b, c);
}
