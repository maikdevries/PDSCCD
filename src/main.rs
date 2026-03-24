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

use pcd::distributed::core::{Graph, Location, Node, Participant, Query};
use std::collections::{HashMap, VecDeque};

fn main() {
    let mut participants = HashMap::from([
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

    let A = participants
        .get_mut("A")
        .expect("Participant must have known ID");

    let external = A
        .graph
        .nodes
        .values()
        .filter(|n| matches!(n.location, Location::External(_)) && n.neighbours.len() > 0)
        .map(|n| Query::new(n.id))
        .collect();

    let (components, candidates) = Participant::compute(&A.graph, external);
    println!("Components: {components:?}");
    println!("Candidates: {candidates:?}");

    let queries = A.send(candidates);
    println!("Queries: {queries:?}");

    println!("---");

    let mut queue = VecDeque::from_iter(queries);

    while let Some((id, queries)) = queue.pop_front() {
        let participant = participants
            .get_mut(id)
            .expect("Participant must have known ID");

        let (resolved, unresolved) = participant.receive(queries);
        println!("Resolved: {resolved:?}");
        println!("Unresolved: {unresolved:?}");

        let (components, candidates) = Participant::compute(&participant.graph, unresolved);
        println!("Components: {components:?}");
        println!("Candidates: {candidates:?}");

        let queries = participant.send(candidates);
        println!("Queries: {queries:?}");

        println!("---");
        queue.extend(queries.into_iter());
    }
}
