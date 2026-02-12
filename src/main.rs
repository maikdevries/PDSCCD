use pcd::graph::{
    core::{Graph, Node},
    johnson::Johnson,
};

fn main() {
    let graph = Graph::new(vec![
        Node::new(1, vec![2]),
        Node::new(2, vec![3]),
        Node::new(3, vec![4]),
        Node::new(4, vec![5]),
        Node::new(5, vec![1]),
    ]);

    Johnson::new(graph).detect();
}
