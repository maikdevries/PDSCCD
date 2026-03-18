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

use pcd::distributed::core::{Graph, Location, Node, Participant, Protocol};

fn main() {
    let A = Participant::new(
        "A",
        Graph::new(vec![
            Node::new(0, Location::External("C"), vec![1]),
            Node::new(1, Location::Internal, vec![2]),
            Node::new(2, Location::Internal, vec![3]),
            Node::new(3, Location::External("B"), vec![]),
        ]),
    );

    let B = Participant::new(
        "B",
        Graph::new(vec![
            Node::new(2, Location::External("A"), vec![3]),
            Node::new(3, Location::Internal, vec![4]),
            Node::new(4, Location::Internal, vec![5]),
            Node::new(5, Location::External("C"), vec![]),
        ]),
    );

    let C = Participant::new(
        "C",
        Graph::new(vec![
            Node::new(4, Location::External("B"), vec![5]),
            Node::new(5, Location::Internal, vec![0]),
            Node::new(0, Location::Internal, vec![1]),
            Node::new(1, Location::External("A"), vec![]),
        ]),
    );

    let protocol = Protocol::new(vec![&A, &B, &C]);
}
