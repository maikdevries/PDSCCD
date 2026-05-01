use curve25519_dalek::Scalar;
use std::collections::{HashMap, HashSet};

use crate::private::{
    crypto::{Ciphertext, Crypto, Sealed, Unsealed},
    tarjan::{Component, Path, Tarjan},
};

#[derive(Clone)]
pub struct Participant<'a> {
    capacity: usize,
    crypto: &'a Crypto,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, Vec<Path>>,
    tokens: HashMap<NID, Vec<Scalar>>,
}

impl<'a> Participant<'a> {
    pub fn new(id: PID, graph: Graph, crypto: &'a Crypto, capacity: usize) -> Self {
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
            .partition(|query| self.tokens.get(&query.target).is_some());
    }

    pub fn detect(&mut self, targets: &HashSet<NID>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).detect(targets);

        // [NOTE]
        self.paths.extend(paths);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        return components.into_iter().filter(|c| c.len() > 1).collect();
    }

    pub fn register(&mut self, nodes: HashSet<NID>) -> Vec<Query> {
        // [NOTE]
        return nodes
            .into_iter()
            .map(|node| {
                let token = Scalar::random(&mut rand::rng());
                self.tokens.entry(node).or_default().push(token);

                return Query {
                    capacity: self.capacity,
                    path: Vec::new(),
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
                if let Some(capacity) = query.capacity.checked_sub(path.nodes.len()) {
                    map.entry(path.participant).or_default().push(Query {
                        capacity: capacity,
                        path: query
                            .path
                            .iter()
                            .chain(
                                &path
                                    .nodes
                                    .iter()
                                    .map(|n| query.token * Scalar::from(*n))
                                    .collect::<Vec<Ciphertext>>(),
                            )
                            .map(|c| self.crypto.rerandomise(c))
                            .collect(),
                        target: path.target,
                        token: self.crypto.rerandomise(&query.token),
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
            let mut bache = HashMap::new();
            let mut qache = HashMap::new();

            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE]
            let seals = queries
                .into_iter()
                .map(|query| {
                    let seal = Sealed {
                        nonce: rand::random::<u128>(),
                        token: query.token * alpha,
                    };

                    qache.insert(seal.nonce, query);
                    return seal;
                })
                .collect();

            // [NOTE]
            let blinds = self
                .tokens
                .get(&node)
                .expect("Target node tokens must be known to decrypt")
                .iter()
                .map(|token| {
                    let nonce = rand::random::<u128>();
                    bache.insert(nonce, token.invert());

                    return Unsealed {
                        nonce: nonce,
                        token: Crypto::encode(*token) * beta,
                    };
                })
                .collect();

            let (unsealed, blinds) = self.crypto.unseal(seals, blinds);

            // [NOTE]
            let blinds: HashMap<[u8; 32], u128> = blinds
                .into_iter()
                .map(|blind| ((blind.token * alpha).compress().to_bytes(), blind.nonce))
                .collect();

            // [NOTE]
            for unseal in unsealed {
                let bytes = (unseal.token * beta).compress().to_bytes();

                if let Some(nonce) = blinds.get(&bytes) {
                    let gamma = Scalar::random(&mut rand::rng());
                    let inverse = bache.get(nonce).expect("Blind nonce must be known");

                    // [NOTE]
                    let component = qache
                        .remove(&unseal.nonce)
                        .expect("Unsealed nonce must be known")
                        .path
                        .into_iter()
                        .map(|c| {
                            self.crypto
                                .recover(
                                    &(self.crypto.decrypt(&(c * gamma)) * gamma.invert() * inverse),
                                )
                                .unwrap()
                        })
                        .collect();

                    components.push(component);
                } else {
                    incomplete.push(
                        qache
                            .remove(&unseal.nonce)
                            .expect("Unsealed nonce must be known"),
                    );
                }
            }
        }

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
