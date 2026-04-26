use std::collections::{HashMap, HashSet, VecDeque};

use crate::private::{
    core::{Location, PID, Participant, Query},
    crypto::Ciphertext,
    tarjan::Component,
};

// ---

#[cfg(debug_assertions)]
macro_rules! debug_println {
    ($( $args:expr ), *) => { println!($( $args ), *) }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_println {
    ($( $args:expr ), *) => {
        ()
    };
}

// ---

pub struct Protocol<'a> {
    participants: HashMap<PID, Participant<'a>>,
    queue: VecDeque<(PID, Vec<Query>)>,
}

impl<'a> Protocol<'a> {
    pub fn new<const N: usize>(participants: [Participant<'a>; N]) -> Self {
        Self {
            participants: participants.into_iter().map(|p| (p.id, p)).collect(),
            queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self, initiator: PID) -> HashMap<PID, Vec<Component>> {
        let participant = self
            .participants
            .get(initiator)
            .expect("Participant must have known ID");

        self.queue
            .push_back((initiator, Protocol::prepare(participant)));

        return self.process();
    }

    fn prepare(participant: &Participant) -> Vec<Query> {
        return participant
            .graph
            .nodes
            .values()
            // [NOTE]
            .fold(HashSet::new(), |mut set, node| {
                if matches!(node.location, Location::External(_)) {
                    set.extend(&node.neighbours);
                }

                return set;
            })
            .into_iter()
            .map(|node| Query {
                capacity: 0,
                from: "",
                path: Vec::new(),
                target: node,
                token: Ciphertext::default(),
            })
            .collect();
    }

    fn process(&mut self) -> HashMap<PID, Vec<Component>> {
        let mut components: HashMap<PID, Vec<Component>> = HashMap::new();

        while let Some((id, mut queries)) = self.queue.pop_front() {
            // [NOTE] Collect all consecutive requests for same participant into single batch
            while let Some((_, next)) = self.queue.pop_front_if(|(next, _)| *next == id) {
                queries.extend(next);
            }

            let participant = self
                .participants
                .get_mut(id)
                .expect("Participant must have known ID");

            debug_println!();
            debug_println!("--- PARTICIPANT {id} START ---");

            // [NOTE]
            let (known, unknown) = participant.receive(queries);
            debug_println!("Known: {known:?}");
            debug_println!("Unknown: {unknown:?}");

            // [NOTE]
            let (complete, incomplete) = participant.decrypt(known);
            debug_println!("Complete: {complete:?}");
            debug_println!("Incomplete: {incomplete:?}");

            let targets = unknown.iter().map(|query| query.target).collect();

            // [NOTE]
            let detected = participant.detect(&targets);
            debug_println!("Detected: {detected:?}");

            // [NOTE]
            let registered = participant.register(targets);
            debug_println!("Registered: {registered:?}");

            // [NOTE]
            let queries = participant.forward(
                incomplete
                    .into_iter()
                    .chain(unknown)
                    .chain(registered)
                    .collect(),
            );
            debug_println!("Queries: {queries:?}");

            debug_println!("--- PARTICIPANT {id} END ---");

            // [NOTE]
            components.entry(id).or_default().extend(complete);
            self.queue.extend(queries);
        }

        return components;
    }
}
