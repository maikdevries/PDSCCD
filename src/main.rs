use std::sync::Arc;

use pdsccd::private::{
    core::{Graph, Location, Node, Participant},
    crypto::Elliptic,
    protocol::Protocol,
};

fn main() {
    let crypto = Arc::new(Elliptic::new());

    let participants = [
        Participant::new(
            "A",
            Graph::new(vec![
                Node::new(0, Location::External("C"), vec![1]),
                Node::new(1, Location::Internal, vec![2]),
                Node::new(2, Location::Internal, vec![3]),
                Node::new(3, Location::Internal, vec![4]),
                Node::new(4, Location::External("B"), vec![]),
                Node::new(6, Location::External("C"), vec![7]),
                Node::new(7, Location::Internal, vec![2]),
            ]),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "B",
            Graph::new(vec![
                Node::new(3, Location::External("A"), vec![4]),
                Node::new(4, Location::Internal, vec![5]),
                Node::new(5, Location::Internal, vec![6]),
                Node::new(6, Location::External("C"), vec![4]),
            ]),
            crypto.clone(),
            6,
        ),
        Participant::new(
            "C",
            Graph::new(vec![
                Node::new(0, Location::Internal, vec![1]),
                Node::new(1, Location::External("A"), vec![]),
                Node::new(4, Location::External("B"), vec![]),
                Node::new(5, Location::External("B"), vec![6]),
                Node::new(6, Location::Internal, vec![4, 7]),
                Node::new(7, Location::External("A"), vec![]),
            ]),
            crypto.clone(),
            6,
        ),
    ]
    .into();

    let time = std::time::Instant::now();
    let components = Protocol::new(participants).run("A");

    println!();
    println!("Components: {components:?}");
    println!("Duration: {:?}", time.elapsed());
}
