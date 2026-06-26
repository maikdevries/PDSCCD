use std::sync::Arc;

use pdsccd::private::{
    core::{Graph, Location, Node, Participant},
    crypto::Elliptic,
    protocol::Protocol,
};

#[test]
fn success() {
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(0, Location::External("C"), vec![1]),
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![3]),
                    Node::new(3, Location::External("B"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(2, Location::External("A"), vec![3]),
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![5]),
                    Node::new(5, Location::External("C"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(4, Location::External("B"), vec![5]),
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![1]),
                    Node::new(1, Location::External("A"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
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
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
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
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![3]),
                    Node::new(3, Location::External("B"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![5]),
                    Node::new(5, Location::External("C"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![1]),
                    Node::new(1, Location::External("A"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
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
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(0, Location::External("C"), vec![1]),
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(2, Location::External("A"), vec![3]),
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(4, Location::External("B"), vec![5]),
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
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
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(0, Location::External("C"), vec![1]),
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![3]),
                    Node::new(3, Location::External("B"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            5,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(2, Location::External("A"), vec![3]),
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![5]),
                    Node::new(5, Location::External("C"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            5,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(4, Location::External("B"), vec![5]),
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![1]),
                    Node::new(1, Location::External("A"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            5,
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
    let crypto = Arc::new(Elliptic::new());

    let participants = vec![
        Participant::new(
            "A",
            Graph::new(
                [
                    Node::new(0, Location::External("C"), vec![1]),
                    Node::new(1, Location::Internal, vec![2]),
                    Node::new(2, Location::Internal, vec![3]),
                    Node::new(3, Location::External("B"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            5,
        ),
        Participant::new(
            "B",
            Graph::new(
                [
                    Node::new(2, Location::External("A"), vec![3]),
                    Node::new(3, Location::Internal, vec![4]),
                    Node::new(4, Location::Internal, vec![5]),
                    Node::new(5, Location::External("C"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(
                [
                    Node::new(4, Location::External("B"), vec![5]),
                    Node::new(5, Location::Internal, vec![0]),
                    Node::new(0, Location::Internal, vec![1]),
                    Node::new(1, Location::External("A"), vec![]),
                ]
                .into(),
            ),
            crypto.clone(),
            7,
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A");
    let b = Protocol::new(participants.clone()).run("B");
    let c = Protocol::new(participants).run("C");

    assert!(a["A"].is_empty() && !a["B"].is_empty() && !a["C"].is_empty());

    assert_eq!(a, b);
    assert_eq!(b, c);
}
