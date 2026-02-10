use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone)]
struct Node {
    id: usize,
    neighbours: BTreeSet<usize>,
}

struct Graph {
    nodes: BTreeMap<usize, Node>,
}

impl Graph {
    fn new(adjacency: Vec<Node>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(|node| (node.id, node)).collect(),
        }
    }

    fn induce(&self, s: usize) -> Self {
        let mut nodes = self.nodes.clone().split_off(&s);

        for node in nodes.values_mut() {
            node.neighbours = node.neighbours.split_off(&s);
        }

        Self { nodes }
    }

    fn subgraph(&self, subset: &Vec<usize>) -> Self {
        let mut nodes = self.nodes.clone();

        nodes.retain(|id, node| {
            node.neighbours.retain(|id| subset.contains(id));
            subset.contains(id)
        });

        Self { nodes }
    }
}

struct Johnson {
    B: HashMap<usize, HashSet<usize>>,
    blocked: HashSet<usize>,
    n: usize,
    s: usize,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Johnson {
    fn new(graph: Graph) -> Self {
        Self {
            B: HashMap::new(),
            blocked: HashSet::new(),
            n: graph.nodes.len(),
            s: 1,
            stack: Vec::new(),
            subgraph: graph,
        }
    }

    fn detect(mut self) {
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

            if let Some((&id, _)) = self.subgraph.nodes.first_key_value() {
                self.s = id;

                for i in self.subgraph.nodes.keys() {
                    self.blocked.remove(i);
                    self.B.remove(i);
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
        self.blocked.insert(v);

        let neighbours = self.subgraph.nodes[&v].neighbours.clone();

        for w in &neighbours {
            if *w == self.s {
                let mut stack = self.stack.clone();
                stack.push(self.s);

                println!("Cycle found: {stack:?}");
                f = true;
            } else if !self.blocked.contains(w) && self.circuit(*w) {
                f = true;
            }
        }

        if f {
            self.unblock(v);
        } else {
            for w in neighbours {
                self.B.entry(w).or_insert(HashSet::new()).insert(v);
            }
        }

        self.stack.pop();
        return f;
    }

    fn unblock(&mut self, u: usize) {
        self.blocked.remove(&u);

        let mut stack: Vec<usize> = self.B.remove(&u).into_iter().flatten().collect();

        while let Some(w) = stack.pop() {
            if self.blocked.remove(&w) {
                stack.extend(self.B.remove(&w).into_iter().flatten());
            }
        }
    }
}

struct Tarjan {
    components: Vec<Vec<usize>>,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Tarjan {
    fn new(graph: Graph) -> Self {
        Self {
            components: Vec::new(),
            i: 0,
            lowlink: HashMap::new(),
            number: HashMap::new(),
            stack: Vec::new(),
            subgraph: graph,
        }
    }

    fn detect(mut self) -> Vec<Vec<usize>> {
        for w in self.subgraph.nodes.keys().copied().collect::<Vec<usize>>() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w);
            }
        }

        return self.components.clone();
    }

    fn strong_connect(&mut self, v: usize) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        for w in self.subgraph.nodes[&v].neighbours.clone() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[&w]));
            } else if self.number[&w] < self.number[&v] && self.stack.contains(&w) {
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.number[&w]));
            }
        }

        if self.lowlink[&v] == self.number[&v] {
            let mut scc = Vec::new();

            while let Some(w) = self.stack.pop_if(|w| self.number[w] >= self.number[&v]) {
                scc.push(w);
            }

            self.components.push(scc);
        }
    }
}

fn main() {
    let graph = Graph::new(vec![
        Node {
            id: 1,
            neighbours: BTreeSet::from([2]),
        },
        Node {
            id: 2,
            neighbours: BTreeSet::from([3]),
        },
        Node {
            id: 3,
            neighbours: BTreeSet::from([4]),
        },
        Node {
            id: 4,
            neighbours: BTreeSet::from([5]),
        },
        Node {
            id: 5,
            neighbours: BTreeSet::from([1]),
        },
    ]);

    Johnson::new(graph).detect();
}
