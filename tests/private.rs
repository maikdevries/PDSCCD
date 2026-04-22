use pcd::private::{
    core::{Graph, Location, Node},
    protocol::Protocol,
};

#[test]
fn no_external_nodes() {
    let participants = [
        (
            "A",
            Graph::new([
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, []),
            ]),
        ),
        (
            "B",
            Graph::new([
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, []),
            ]),
        ),
        (
            "C",
            Graph::new([
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A", 6);
    let b = Protocol::new(participants.clone()).run("B", 6);
    let c = Protocol::new(participants).run("C", 6);

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn no_external_incoming() {
    let participants = [
        (
            "A",
            Graph::new([
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        (
            "B",
            Graph::new([
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        (
            "C",
            Graph::new([
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A", 6);
    let b = Protocol::new(participants.clone()).run("B", 6);
    let c = Protocol::new(participants).run("C", 6);

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn no_external_outgoing() {
    let participants = [
        (
            "A",
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, []),
            ]),
        ),
        (
            "B",
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, []),
            ]),
        ),
        (
            "C",
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A", 6);
    let b = Protocol::new(participants.clone()).run("B", 6);
    let c = Protocol::new(participants).run("C", 6);

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}

#[test]
fn time_to_live_exceeded() {
    let participants = [
        (
            "A",
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        (
            "B",
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        (
            "C",
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let a = Protocol::new(participants.clone()).run("A", 5);
    let b = Protocol::new(participants.clone()).run("B", 5);
    let c = Protocol::new(participants).run("C", 5);

    assert!(a.values().all(|components| components.is_empty()));
    assert!(b.values().all(|components| components.is_empty()));
    assert!(c.values().all(|components| components.is_empty()));
}
