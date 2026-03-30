use crate::distributed::tarjan::Tarjan;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Participant {
    pub graph: Graph,
    seen: BTreeMap<usize, BTreeSet<u128>>,
}

impl Participant {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            seen: BTreeMap::new(),
        }
    }

    pub fn compute(
        graph: &Graph,
        queries: Vec<Query>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        let (components, candidates) = Tarjan::new(graph).detect(queries);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        let components = components.into_iter().filter(|c| c.len() > 1).collect();

        return (components, candidates);
    }

    pub fn receive(&self, queries: Vec<Query>) -> (Vec<Query>, Vec<Query>) {
        return queries
            .into_iter()
            // [NOTE] Combine queries with same target and token into single query
            .fold(HashMap::new(), |mut map, query| {
                map.entry((query.target, query.token))
                    .and_modify(|existing: &mut Query| existing.nodes.extend(query.nodes.clone()))
                    .or_insert(query);

                return map;
            })
            .into_values()
            .partition(|query| {
                self.seen
                    .get(&query.target)
                    .and_then(|tokens| tokens.get(&query.token))
                    .is_some()
            });
    }

    pub fn send(
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
                        .inspect(|candidate| {
                            self.seen
                                .entry(candidate.source)
                                .or_insert(BTreeSet::new())
                                .insert(candidate.token);
                        })
                        // [NOTE] Combine candidates with same target and token into single query
                        .fold(HashMap::new(), |mut map, candidate| {
                            map.entry((candidate.target, candidate.token))
                                .and_modify(|query: &mut Query| {
                                    query.nodes.extend(candidate.nodes.clone())
                                })
                                .or_insert_with(|| Query::from(&candidate));

                            return map;
                        })
                        .into_values()
                        .collect(),
                )
            })
            .collect();
    }
}

pub struct Query {
    pub nodes: BTreeSet<usize>,
    pub target: usize,
    token: u128,
}

impl Query {
    pub fn new(target: usize, token: u128) -> Self {
        Self {
            nodes: BTreeSet::new(),
            target,
            token,
        }
    }
}

impl From<&Candidate> for Query {
    fn from(candidate: &Candidate) -> Self {
        Self {
            nodes: candidate.nodes.clone(),
            target: candidate.target,
            token: candidate.token,
        }
    }
}

pub struct Candidate {
    nodes: BTreeSet<usize>,
    source: usize,
    target: usize,
    token: u128,
}

impl Candidate {
    pub fn from(query: &Query, target: usize, path: &Vec<usize>) -> Self {
        Self {
            nodes: query.nodes.iter().chain(path).copied().collect(),
            source: query.target,
            target: target,
            token: query.token,
        }
    }
}

pub struct Graph {
    pub nodes: BTreeMap<usize, Node>,
}

impl Graph {
    pub fn new<const N: usize>(nodes: [Node; N]) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    id: usize,
    pub location: Location,
    pub neighbours: BTreeSet<usize>,
}

impl Node {
    pub fn new<const N: usize>(id: usize, location: Location, neighbours: [usize; N]) -> Self {
        Self {
            id,
            location,
            neighbours: neighbours.into(),
        }
    }
}

pub enum Location {
    External(&'static str),
    Internal,
}
