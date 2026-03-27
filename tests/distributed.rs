use pcd::distributed::{
    core::{Graph, Location, Node, Participant},
    protocol::Protocol,
};
use std::collections::HashMap;

#[test]
fn one_one_one() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![1]),
                Node::new(1, Location::External("B"), vec![]),
                Node::new(2, Location::External("C"), vec![0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![1]),
                Node::new(1, Location::Internal, vec![2]),
                Node::new(2, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("B"), vec![2]),
                Node::new(2, Location::Internal, vec![0]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_one_two() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![1, 2]),
                Node::new(1, Location::External("B"), vec![]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("C"), vec![0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![1, 2]),
                Node::new(1, Location::Internal, vec![3]),
                Node::new(2, Location::Internal, vec![3]),
                Node::new(3, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("B"), vec![3]),
                Node::new(2, Location::External("B"), vec![3]),
                Node::new(3, Location::Internal, vec![0]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_two_one() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![2]),
                Node::new(1, Location::Internal, vec![2]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("C"), vec![0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![2]),
                Node::new(1, Location::External("A"), vec![2]),
                Node::new(2, Location::Internal, vec![3]),
                Node::new(3, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("A"), vec![]),
                Node::new(2, Location::External("B"), vec![3]),
                Node::new(3, Location::Internal, vec![0, 1]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_two_two() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![2, 3]),
                Node::new(1, Location::Internal, vec![2, 3]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("B"), vec![]),
                Node::new(4, Location::External("C"), vec![0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![2, 3]),
                Node::new(1, Location::External("A"), vec![2, 3]),
                Node::new(2, Location::Internal, vec![4]),
                Node::new(3, Location::Internal, vec![4]),
                Node::new(4, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("A"), vec![]),
                Node::new(2, Location::External("B"), vec![4]),
                Node::new(3, Location::External("B"), vec![4]),
                Node::new(4, Location::Internal, vec![0, 1]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_one_one() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![1]),
                Node::new(1, Location::External("B"), vec![]),
                Node::new(2, Location::External("C"), vec![0]),
                Node::new(3, Location::External("C"), vec![0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![1]),
                Node::new(1, Location::Internal, vec![2, 3]),
                Node::new(2, Location::External("C"), vec![]),
                Node::new(3, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("B"), vec![2, 3]),
                Node::new(2, Location::Internal, vec![0]),
                Node::new(3, Location::Internal, vec![0]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_one_two() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![1, 2]),
                Node::new(1, Location::External("B"), vec![]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("C"), vec![0]),
                Node::new(4, Location::External("C"), vec![0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![1, 2]),
                Node::new(1, Location::Internal, vec![3, 4]),
                Node::new(2, Location::Internal, vec![3, 4]),
                Node::new(3, Location::External("C"), vec![]),
                Node::new(4, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("B"), vec![3, 4]),
                Node::new(2, Location::External("B"), vec![3, 4]),
                Node::new(3, Location::Internal, vec![0]),
                Node::new(4, Location::Internal, vec![0]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_two_one() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![2]),
                Node::new(1, Location::Internal, vec![2]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("C"), vec![0, 1]),
                Node::new(4, Location::External("C"), vec![0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![2]),
                Node::new(1, Location::External("A"), vec![2]),
                Node::new(2, Location::Internal, vec![3, 4]),
                Node::new(3, Location::External("C"), vec![]),
                Node::new(4, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("A"), vec![]),
                Node::new(2, Location::External("B"), vec![3, 4]),
                Node::new(3, Location::Internal, vec![0, 1]),
                Node::new(4, Location::Internal, vec![0, 1]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_two_two() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::Internal, vec![2, 3]),
                Node::new(1, Location::Internal, vec![2, 3]),
                Node::new(2, Location::External("B"), vec![]),
                Node::new(3, Location::External("B"), vec![]),
                Node::new(4, Location::External("C"), vec![0, 1]),
                Node::new(5, Location::External("C"), vec![0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![2, 3]),
                Node::new(1, Location::External("A"), vec![2, 3]),
                Node::new(2, Location::Internal, vec![4, 5]),
                Node::new(3, Location::Internal, vec![4, 5]),
                Node::new(4, Location::External("C"), vec![]),
                Node::new(5, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("A"), vec![]),
                Node::new(1, Location::External("A"), vec![]),
                Node::new(2, Location::External("B"), vec![4, 5]),
                Node::new(3, Location::External("B"), vec![4, 5]),
                Node::new(4, Location::Internal, vec![0, 1]),
                Node::new(5, Location::Internal, vec![0, 1]),
            ])),
        ),
    ]);

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}
