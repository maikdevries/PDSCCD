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

    pub fn receive(&self, messages: Vec<Message>) -> (Vec<Message>, Vec<Message>) {
        // [NOTE]
        return messages
            .into_iter()
            .partition(|message| self.tokens.contains_key(&message.target));
    }

    pub fn detect(&mut self, targets: &HashSet<NID>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).detect(targets);

        // [NOTE]
        for (k, v) in paths {
            self.paths.entry(k).or_default().extend(v);
        }

        return components;
    }

    pub fn register(&mut self, nodes: HashSet<NID>) -> Vec<Message> {
        // [NOTE]
        return nodes
            .into_iter()
            .map(|node| {
                let token = Scalar::random(&mut rand::rng());
                self.tokens.insert(node, token);

                return Message {
                    capacity: self.capacity,
                    nodes: Vec::new(),
                    source: node,
                    target: node,
                    token: self.crypto.encrypt(&Crypto::encode(token)),
                };
            })
            .collect();
    }

    pub fn forward(&self, messages: Vec<Message>) -> HashMap<PID, Vec<Message>> {
        return messages
            .into_iter()
            .fold(HashMap::new(), |mut map, message| {
                // [NOTE]
                for path in self.paths.get(&message.target).into_iter().flatten() {
                    // [NOTE]
                    if path.target != message.source
                        && let Some(capacity) = message.capacity.checked_sub(path.nodes.len())
                        && let Location::External(participant) =
                            self.graph.nodes[&path.target].location
                    {
                        // [NOTE]
                        let token = self.crypto.rerandomise(&message.token);

                        map.entry(participant).or_default().push(Message {
                            capacity: capacity,
                            nodes: message
                                .nodes
                                .iter()
                                .map(|c| self.crypto.rerandomise(c))
                                .chain(path.nodes.iter().map(|n| token * Scalar::from(*n)))
                                .collect(),
                            source: path.exit,
                            target: path.target,
                            token: token,
                        });
                    }
                }

                return map;
            });
    }

    pub fn decrypt(&self, messages: Vec<Message>) -> (HashMap<NID, Component>, Vec<Message>) {
        // [NOTE]
        let groups: HashMap<NID, Vec<Message>> =
            messages
                .into_iter()
                .fold(HashMap::new(), |mut map, message| {
                    map.entry(message.target).or_default().push(message);
                    return map;
                });

        let mut components: HashMap<NID, Component> = HashMap::new();
        let mut incomplete = Vec::new();

        for (node, queries) in groups {
            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE]
            let mut cache = HashMap::with_capacity(queries.len());
            let seals = queries
                .into_iter()
                .map(|message| {
                    let seal = Sealed {
                        nonce: rand::random::<u128>(),
                        token: message.token * alpha,
                    };

                    cache.insert(seal.nonce, message);
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
                let message = cache.remove(&unseal.nonce).unwrap();

                if unseal.token * beta == blind {
                    let gamma = Scalar::random(&mut rand::rng());

                    // [NOTE]
                    let component: Component = message
                        .nodes
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

                    components.entry(node).or_default().extend(component);
                } else {
                    incomplete.push(message);
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
pub struct Message {
    pub capacity: usize,
    pub nodes: Vec<Ciphertext>,
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
