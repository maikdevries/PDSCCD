use curve25519_dalek::Scalar;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::private::{
    crypto::{Ciphertext, Crypto, Sealed},
    tarjan::{Component, Path, Tarjan},
};

#[derive(Clone)]
pub struct Participant {
    capacity: usize,
    crypto: Arc<Crypto>,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, Vec<Path>>,
    tokens: HashMap<NID, Scalar>,
}

impl Participant {
    pub fn new(id: PID, graph: Graph, crypto: Arc<Crypto>, capacity: usize) -> Self {
        Self {
            capacity: capacity,
            crypto: crypto,
            graph: graph,
            id: id,
            paths: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn receive(&self, queries: Vec<Query>) -> (Vec<Query>, Vec<Query>) {
        // [NOTE]
        return queries
            .into_iter()
            .partition(|query| self.tokens.contains_key(&query.target));
    }

    pub fn detect(&mut self, targets: &HashSet<NID>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).detect(targets);

        // [NOTE]
        self.paths.extend(paths);

        return components;
    }

    pub fn register(&mut self, nodes: HashSet<NID>) -> Vec<Query> {
        // [NOTE]
        return nodes
            .into_iter()
            .map(|node| {
                let token = Scalar::random(&mut rand::rng());
                self.tokens.insert(node, token);

                return Query {
                    capacity: self.capacity,
                    path: Vec::new(),
                    source: node,
                    target: node,
                    token: self.crypto.encrypt(&Crypto::encode(token)),
                };
            })
            .collect();
    }

    pub fn forward(&self, queries: Vec<Query>) -> HashMap<PID, Vec<Query>> {
        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            // [NOTE]
            for path in self.paths.get(&query.target).into_iter().flatten() {
                // [NOTE]
                if (path.target != query.source || path.nodes.len() > 1)
                    && let Some(capacity) = query.capacity.checked_sub(path.nodes.len())
                    && let Location::External(participant) = self.graph.nodes[&path.target].location
                {
                    // [NOTE]
                    let token = self.crypto.rerandomise(&query.token);

                    map.entry(participant).or_default().push(Query {
                        capacity: capacity,
                        path: query
                            .path
                            .iter()
                            .map(|c| self.crypto.rerandomise(c))
                            .chain(path.nodes.iter().map(|n| token * Scalar::from(*n)))
                            .collect(),
                        source: *path.nodes.last().unwrap(),
                        target: path.target,
                        token: token,
                    });
                }
            }

            return map;
        });
    }

    pub fn decrypt(&self, queries: Vec<Query>) -> (Vec<Component>, Vec<Query>) {
        // [NOTE]
        let groups: HashMap<NID, Vec<Query>> =
            queries.into_iter().fold(HashMap::new(), |mut map, query| {
                map.entry(query.target).or_default().push(query);
                return map;
            });

        let mut components = Vec::new();
        let mut incomplete = Vec::new();

        for (node, queries) in groups {
            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE]
            let mut cache = HashMap::with_capacity(queries.len());
            let seals = queries
                .into_iter()
                .map(|query| {
                    let seal = Sealed {
                        nonce: rand::random::<u128>(),
                        token: query.token * alpha,
                    };

                    cache.insert(seal.nonce, query);
                    return seal;
                })
                .collect();

            // [NOTE]
            let token = self.tokens.get(&node).unwrap();
            let blind = Crypto::encode(*token) * beta;

            // [NOTE]
            let (unsealed, blind) = self.crypto.unseal(seals, blind);
            let blind = blind * alpha;

            // [NOTE]
            for unseal in unsealed {
                let query = cache.remove(&unseal.nonce).unwrap();

                if unseal.token * beta == blind {
                    let gamma = Scalar::random(&mut rand::rng());

                    // [NOTE]
                    let component = query
                        .path
                        .into_iter()
                        .map(|c| {
                            self.crypto
                                .recover(
                                    &(self.crypto.decrypt(&(c * gamma))
                                        * gamma.invert()
                                        * token.invert()),
                                )
                                .unwrap()
                        })
                        .collect();

                    components.push(component);
                } else {
                    incomplete.push(query);
                }
            }
        }

        // [NOTE]
        components.retain(|c: &Component| {
            let mut seen = HashSet::new();
            return c.iter().all(|x| seen.insert(x));
        });

        return (components, incomplete);
    }
}

pub type NID = u32;
pub type PID = &'static str;

// [TODO]
#[derive(Debug)]
pub struct Query {
    pub capacity: usize,
    pub path: Vec<Ciphertext>,
    pub source: NID,
    pub target: NID,
    pub token: Ciphertext,
}

#[derive(Clone)]
pub struct Graph {
    pub nodes: HashMap<NID, Node>,
}

impl Graph {
    pub fn new<const N: usize>(nodes: [Node; N]) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    id: NID,
    pub location: Location,
    pub neighbours: Vec<NID>,
}

impl Node {
    pub fn new<const N: usize>(id: NID, location: Location, neighbours: [NID; N]) -> Self {
        Self {
            id: id,
            location: location,
            neighbours: neighbours.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Location {
    External(PID),
    Internal,
}
