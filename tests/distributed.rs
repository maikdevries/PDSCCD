use pdsccd::distributed::{
    core::{Graph, Location, Node, Participant},
    protocol::Protocol,
};

#[test]
fn one_one_one() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("B"), []),
                Node::new(2, Location::External("C"), [0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("B"), [2]),
                Node::new(2, Location::Internal, [0]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_one_two() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [1, 2]),
                Node::new(1, Location::External("B"), []),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("C"), [0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [1, 2]),
                Node::new(1, Location::Internal, [3]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("B"), [3]),
                Node::new(2, Location::External("B"), [3]),
                Node::new(3, Location::Internal, [0]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_two_one() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [2]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("C"), [0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [2]),
                Node::new(1, Location::External("A"), [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("A"), []),
                Node::new(2, Location::External("B"), [3]),
                Node::new(3, Location::Internal, [0, 1]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn one_two_two() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [2, 3]),
                Node::new(1, Location::Internal, [2, 3]),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("B"), []),
                Node::new(4, Location::External("C"), [0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [2, 3]),
                Node::new(1, Location::External("A"), [2, 3]),
                Node::new(2, Location::Internal, [4]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("A"), []),
                Node::new(2, Location::External("B"), [4]),
                Node::new(3, Location::External("B"), [4]),
                Node::new(4, Location::Internal, [0, 1]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_one_one() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("B"), []),
                Node::new(2, Location::External("C"), [0]),
                Node::new(3, Location::External("C"), [0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [1]),
                Node::new(1, Location::Internal, [2, 3]),
                Node::new(2, Location::External("C"), []),
                Node::new(3, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("B"), [2, 3]),
                Node::new(2, Location::Internal, [0]),
                Node::new(3, Location::Internal, [0]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_one_two() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [1, 2]),
                Node::new(1, Location::External("B"), []),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("C"), [0]),
                Node::new(4, Location::External("C"), [0]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [1, 2]),
                Node::new(1, Location::Internal, [3, 4]),
                Node::new(2, Location::Internal, [3, 4]),
                Node::new(3, Location::External("C"), []),
                Node::new(4, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("B"), [3, 4]),
                Node::new(2, Location::External("B"), [3, 4]),
                Node::new(3, Location::Internal, [0]),
                Node::new(4, Location::Internal, [0]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_two_one() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [2]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("C"), [0, 1]),
                Node::new(4, Location::External("C"), [0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [2]),
                Node::new(1, Location::External("A"), [2]),
                Node::new(2, Location::Internal, [3, 4]),
                Node::new(3, Location::External("C"), []),
                Node::new(4, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("A"), []),
                Node::new(2, Location::External("B"), [3, 4]),
                Node::new(3, Location::Internal, [0, 1]),
                Node::new(4, Location::Internal, [0, 1]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn two_two_two() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::Internal, [2, 3]),
                Node::new(1, Location::Internal, [2, 3]),
                Node::new(2, Location::External("B"), []),
                Node::new(3, Location::External("B"), []),
                Node::new(4, Location::External("C"), [0, 1]),
                Node::new(5, Location::External("C"), [0, 1]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), [2, 3]),
                Node::new(1, Location::External("A"), [2, 3]),
                Node::new(2, Location::Internal, [4, 5]),
                Node::new(3, Location::Internal, [4, 5]),
                Node::new(4, Location::External("C"), []),
                Node::new(5, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(0, Location::External("A"), []),
                Node::new(1, Location::External("A"), []),
                Node::new(2, Location::External("B"), [4, 5]),
                Node::new(3, Location::External("B"), [4, 5]),
                Node::new(4, Location::Internal, [0, 1]),
                Node::new(5, Location::Internal, [0, 1]),
            ])),
        ),
    ]
    .into();

    let mut protocol = Protocol::new(participants);

    let a = protocol.run("A");
    let b = protocol.run("B");
    let c = protocol.run("C");

    assert_eq!(a, b);
    assert_eq!(b, c);
}
