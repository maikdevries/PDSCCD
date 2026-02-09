#[derive(Clone)]
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

    fn induce(&self, s: usize) -> Self {
        Self {
            nodes: self
                .nodes
                .iter()
                .filter(|node| node.id >= s)
                .map(|node| {
                    Node::new(
                        node.id,
                        node.neighbours
                            .iter()
                            .filter(|&&id| id >= s)
                            .copied()
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    fn subgraph(&self, nodes: &Vec<usize>) -> Self {
        Self {
            nodes: self
                .nodes
                .iter()
                .filter(|node| nodes.contains(&node.id))
                .map(|node| {
                    Node::new(
                        node.id,
                        node.neighbours
                            .iter()
                            .filter(|id| nodes.contains(id))
                            .copied()
                            .collect(),
                    )
                })
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
            // [NOTE] Compute strongest connected component of subgraph G induced by { s, s + 1, ..., n }
            self.subgraph = {
                let components = Tarjan::new(self.subgraph.induce(self.s)).detect();
                let component = components.iter().min_by_key(|c| c.iter().min());

                if let Some(scc) = component {
                    self.subgraph.subgraph(scc)
                } else {
                    Graph::new(Vec::new())
                }
            };

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

struct Tarjan {
    components: Vec<Vec<usize>>,
    i: usize,
    lowlink: Vec<usize>,
    number: Vec<usize>,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Tarjan {
    fn new(graph: Graph) -> Self {
        Self {
            components: Vec::new(),
            i: 0,
            lowlink: vec![0; graph.nodes.len()],
            number: vec![0; graph.nodes.len()],
            stack: Vec::new(),
            subgraph: graph,
        }
    }

    fn detect(&mut self) -> Vec<Vec<usize>> {
        for w in self.subgraph.nodes.clone() {
            if self.number[w.id] == 0 {
                self.strong_connect(w.id);
            }
        }

        return self.components.clone();
    }

    fn strong_connect(&mut self, v: usize) {
        self.i += 1;
        self.lowlink[v] = self.i;
        self.number[v] = self.i;

        self.stack.push(v);

        let neighbours = self.subgraph.nodes[v].neighbours.clone();

        for w in neighbours {
            if self.number[w] == 0 {
                // (v, w) is a tree arc
                self.strong_connect(w);
                self.lowlink[v] = self.lowlink[v].min(self.lowlink[w]);
            } else if self.number[w] < self.number[v] {
                // (v, w) is a frond or cross-link
                if self.stack.contains(&w) {
                    self.lowlink[v] = self.lowlink[v].min(self.number[w]);
                }
            }
        }

        if self.lowlink[v] == self.number[v] {
            // v is the root of a component
            let mut scc = Vec::new();

            while let Some(w) = self.stack.pop_if(|w| self.number[*w] >= self.number[v]) {
                scc.push(w);
            }

            self.components.push(scc);
        }
    }
}
