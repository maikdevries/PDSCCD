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
            // [BUG] Last graph node might have ID higher than number of nodes in graph
            n: graph.nodes.len(),
            // [BUG] First graph node might have ID other than 1
            s: 1,
            stack: Vec::new(),
            // [PERF] Initialise to empty Graph to avoid expensive clone of input graph
            subgraph: graph,
        }
    }

    pub fn detect(mut self) {
        // [BUG] Does not consider components rooted in last graph node
        while self.s < self.n {
            // [NOTE] Compute strongest connected component of subgraph G induced by { s, s + 1, ..., n }
            self.subgraph = {
                // [PERF] Induce graph from previous induced graph
                let components = Tarjan::new(self.graph.induce(self.s)).detect();

                // [BUG] Strongly connected components might not include starting node
                let component = components.into_iter().find(|c| c.contains(&self.s));

                if let Some(scc) = component {
                    // [PERF] Extract subgraph from induced graph
                    self.graph.subgraph(&scc)
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

        // [PERF] Use reference to avoid expensive clone of neighbours
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
