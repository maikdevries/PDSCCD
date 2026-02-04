struct Node {
    neighbours: Vec<usize>,
}

impl Node {
    fn new(neighbours: Vec<usize>) -> Self {
        Self { neighbours }
    }
}

struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    fn new(adjacency: Vec<Vec<usize>>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(Node::new).collect(),
        }
    }
}

fn main() {
    let graph = Graph::new(vec![vec![1], vec![2], vec![3], vec![4], vec![0]]);
}
