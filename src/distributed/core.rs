use crate::distributed::tarjan::{Candidate, Tarjan};
use std::collections::{BTreeMap, BTreeSet, HashMap};

struct Protocol {
    out: HashMap<usize, HashMap<u128, Candidate>>,
}

impl Protocol {
    fn new() -> Self {
        Self {
            out: HashMap::new(),
        }
    }

    fn compute(
        graph: &Graph,
        queries: Vec<Query>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        let (components, candidates) = Tarjan::new(graph).detect(queries);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        let components = components.into_iter().filter(|c| c.len() > 1).collect();

        return (components, candidates);
    }

    fn prepare(
        &mut self,
        candidates: HashMap<&'static str, Vec<Candidate>>,
    ) -> HashMap<&'static str, Vec<Query>> {
        return candidates
            .into_iter()
            .map(|(participant, candidates)| {
                (
                    participant,
                    candidates
                        .into_iter()
                        .map(|c| {
                            let query = Query::new(
                                *c.path.last().expect("Candidate path must not be empty"),
                                c.token,
                            );

                            self.out
                                .entry(c.source)
                                .or_insert(HashMap::new())
                                .insert(c.token, c);

                            return query;
                        })
                        .collect(),
                )
            })
            .collect();
    }

    fn resolve(&self, mut queries: Vec<Query>) -> (Vec<&Candidate>, Vec<Query>) {
        let mut candidates = Vec::new();

        queries.retain(|query| {
            if let Some(candidate) = self
                .out
                .get(&query.node)
                .and_then(|tokens| tokens.get(&query.token.expect("Query must contain a token")))
            {
                candidates.push(candidate);
                return false;
            } else {
                return true;
            }
        });

        return (candidates, queries);
    }
}

pub struct Participant {
    id: &'static str,
    pub graph: Graph,
    protocol: Protocol,
}

impl Participant {
    pub fn new(id: &'static str, graph: Graph) -> Self {
        Self {
            id,
            graph,
            protocol: Protocol::new(),
        }
    }

    pub fn compute(
        &self,
        nodes: Vec<Query>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        return Protocol::compute(&self.graph, nodes);
    }

    pub fn receive(&self, queries: Vec<Query>) -> (Vec<&Candidate>, Vec<Query>) {
        return self.protocol.resolve(queries);
    }

    pub fn send(
        &mut self,
        candidates: HashMap<&'static str, Vec<Candidate>>,
    ) -> HashMap<&'static str, Vec<Query>> {
        return self.protocol.prepare(candidates);
    }
}

#[derive(Debug)]
pub struct Query {
    pub node: usize,
    pub token: Option<u128>,
}

impl Query {
    fn new(node: usize, token: u128) -> Self {
        Self {
            node,
            token: Some(token),
        }
    }
}

pub struct Graph {
    pub nodes: BTreeMap<usize, Node>,
}

impl Graph {
    pub fn new(adjacency: Vec<Node>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    pub id: usize,
    pub location: Location,
    pub neighbours: BTreeSet<usize>,
}

impl Node {
    pub fn new(id: usize, location: Location, neighbours: Vec<usize>) -> Self {
        Self {
            id,
            location,
            neighbours: BTreeSet::from_iter(neighbours),
        }
    }
}

pub enum Location {
    External(&'static str),
    Internal,
}
