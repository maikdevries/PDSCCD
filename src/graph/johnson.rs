use std::collections::{HashMap, HashSet};

use crate::graph::{core::Graph, tarjan::Tarjan};

pub struct Johnson {
    B: HashMap<usize, HashSet<usize>>,
    blocked: HashSet<usize>,
    graph: Graph,
    n: usize,
    s: usize,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Johnson {
    pub fn new(graph: Graph) -> Self {
        Self {
            B: HashMap::new(),
            blocked: HashSet::new(),
            graph: graph.clone(),
            n: graph.nodes.len(),
            s: 1,
            stack: Vec::new(),
            subgraph: graph,
        }
    }

    pub fn detect(mut self) {
        while self.s < self.n {
            // [NOTE] Compute strongest connected component of subgraph G induced by { s, s + 1, ..., n }
            self.subgraph = {
                let components = Tarjan::new(self.graph.induce(self.s)).detect();
                let component = components.iter().min_by_key(|c| c.iter().min());

                if let Some(scc) = component {
                    self.graph.subgraph(scc)
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
