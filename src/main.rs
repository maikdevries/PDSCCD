// use pcd::centralised::{
//     core::{Graph, Node},
//     johnson::Johnson,
// };

// fn main() {
//     let graph = Graph::new(vec![
//         Node::new(1, vec![2]),
//         Node::new(2, vec![3]),
//         Node::new(3, vec![4]),
//         Node::new(4, vec![5]),
//         Node::new(5, vec![1]),
//     ]);

//     Johnson::new(graph).detect();
// }

// --- --- --- --- ---

use pcd::distributed::{
    core::{Graph, Location, Node, Participant},
    protocol::Protocol,
};
use std::collections::HashMap;

fn main() {
    let participants = HashMap::from([
        (
            "A",
            Participant::new(Graph::new(vec![
                Node::new(0, Location::External("C"), vec![1]),
                Node::new(1, Location::Internal, vec![2]),
                Node::new(2, Location::Internal, vec![3]),
                Node::new(3, Location::External("B"), vec![]),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new(vec![
                Node::new(2, Location::External("A"), vec![3]),
                Node::new(3, Location::Internal, vec![4]),
                Node::new(4, Location::Internal, vec![5]),
                Node::new(5, Location::External("C"), vec![]),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new(vec![
                Node::new(4, Location::External("B"), vec![5]),
                Node::new(5, Location::Internal, vec![0]),
                Node::new(0, Location::Internal, vec![1]),
                Node::new(1, Location::External("A"), vec![]),
            ])),
        ),
    ]);

    Protocol::new(participants).run("A");
}
