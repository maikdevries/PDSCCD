struct Node {
    id: usize,
    neighbours: Vec<usize>,
}

impl Node {
    fn new(id: usize, neighbours: Vec<usize>) -> Self {
        Self { id, neighbours }
    }
}

struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    fn new(adjacency: Vec<Vec<usize>>) -> Self {
        Self {
            nodes: adjacency
                .into_iter()
                .enumerate()
                .map(|(id, neighbours)| Node::new(id, neighbours))
                .collect(),
        }
    }
}

fn main() {
    let graph = Graph::new(vec![vec![1], vec![2], vec![3], vec![4], vec![0]]);

    let mut johnson = Johnson::new(graph);
    johnson.detect();
}

struct Johnson {
    B: Vec<Vec<usize>>,
    blocked: Vec<bool>,
    n: usize,
    s: usize,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Johnson {
    fn new(graph: Graph) -> Self {
        Self {
            B: vec![Vec::new(); graph.nodes.len()],
            blocked: vec![false; graph.nodes.len()],
            n: graph.nodes.len(),
            s: 0,
            stack: Vec::new(),
            subgraph: graph,
        }
    }

    fn detect(&mut self) {
        while self.s < self.n {
            // [TODO] Compute strongest connected component of subgraph G induced by { s, s + 1, ..., n }

            if let Some(node) = self.subgraph.nodes.first() {
                self.s = node.id;

                for node in &self.subgraph.nodes {
                    self.blocked[node.id] = false;
                    self.B[node.id].clear();
                }

                self.circuit(self.s);
                self.s += 1;
            } else {
                self.s = self.n;
            }
        }
    }

    fn circuit(&mut self, v: usize) -> bool {
        let mut f = false;

        self.stack.push(v);
        self.blocked[v] = true;

        let neighbours = self.subgraph.nodes[v].neighbours.clone();

        for &w in &neighbours {
            if w == self.s {
                let mut stack = self.stack.clone();
                stack.push(self.s);

                println!("Cycle found: {stack:?}");
                f = true;
            } else if !self.blocked[w] && self.circuit(w) {
                f = true;
            }
        }

        if f {
            self.unblock(v);
        } else {
            for &w in &neighbours {
                if !self.B[w].contains(&v) {
                    self.B[w].push(v);
                }
            }
        }

        self.stack.pop();
        return f;
    }

    fn unblock(&mut self, u: usize) {
        self.blocked[u] = false;

        let mut stack: Vec<usize> = self.B[u].drain(..).collect();

        while let Some(w) = stack.pop() {
            if self.blocked[w] {
                self.blocked[w] = false;
                stack.extend(self.B[w].drain(..));
            }
        }
    }
}
