use curve25519_dalek::Scalar;
use std::collections::{HashMap, HashSet};

use crate::private::{
    crypto::{Crypto, elliptic, paillier},
    tarjan::{Component, Path, Tarjan},
};

#[derive(Clone)]
pub struct Participant<'a> {
    capacity: usize,
    crypto: &'a Crypto,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, Vec<Path>>,
    tokens: HashMap<NID, Vec<elliptic::Plaintext>>,
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
                let message = elliptic::Elliptic::encode(rand::random::<u128>());
                self.tokens.entry(node).or_default().push(message);

                return Query {
                    capacity: self.capacity,
                    from: self.id,
                    path: Vec::new(),
                    target: node,
                    token: self.crypto.elliptic.encrypt(&message),
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
                    let nodes: Vec<paillier::Ciphertext> = path
                        .nodes
                        .iter()
                        .map(|n| self.crypto.paillier.encrypt(&paillier::Plaintext::from(*n)))
                        .collect();

                    map.entry(path.participant).or_default().push(Query {
                        capacity: capacity,
                        from: self.id,
                        path: query.path.iter().chain(&nodes).copied().collect(),
                        target: path.target,
                        token: self.crypto.elliptic.rerandomise(&query.token),
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
            let mut cache = HashMap::new();

            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE]
            let seals = queries
                .into_iter()
                .map(|query| {
                    let seal = elliptic::Sealed {
                        token: query.token * alpha,
                        nonce: rand::random::<u128>(),
                    };

                    cache.insert(seal.nonce, query);
                    return seal;
                })
                .collect();

            // [NOTE]
            let blinds = self
                .tokens
                .get(&node)
                .expect("Target node tokens must be known to decrypt")
                .iter()
                .map(|token| *token * beta)
                .collect();

            let (unsealed, blinds) = self.crypto.elliptic.unseal(seals, blinds);

            // [NOTE]
            let blinds: HashSet<[u8; 32]> = blinds
                .into_iter()
                .map(|blind| (blind * alpha).compress().to_bytes())
                .collect();

            // [NOTE]
            for unseal in unsealed {
                let bytes = (unseal.token * beta).compress().to_bytes();

                if blinds.contains(&bytes) {
                    let component = cache
                        .remove(&unseal.nonce)
                        .expect("Unsealed nonce must be known")
                        .path
                        .into_iter()
                        .map(|node| {
                            NID::from_le_bytes(
                                self.crypto.paillier.decrypt(&node).to_le_bytes()
                                    [..size_of::<NID>()]
                                    .try_into()
                                    .unwrap(),
                            )
                        })
                        .collect();

                    components.push(component);
                } else {
                    incomplete.push(
                        cache
                            .remove(&unseal.nonce)
                            .expect("Unsealed nonce must be known"),
                    );
                }
            }
        }

        return (components, incomplete);
    }
}

pub type NID = usize;
pub type PID = &'static str;

// [TODO]
#[derive(Debug)]
pub struct Query {
    pub capacity: usize,
    pub from: PID,
    pub path: Vec<paillier::Ciphertext>,
    pub target: NID,
    pub token: elliptic::Ciphertext,
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
