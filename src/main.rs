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

fn main() {
    let participants = [
        (
            "A",
            Participant::new(Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ])),
        ),
        (
            "B",
            Participant::new(Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ])),
        ),
        (
            "C",
            Participant::new(Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ])),
        ),
    ]
    .into();

    Protocol::new(participants).run("A");
}
