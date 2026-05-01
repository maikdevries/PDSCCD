use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

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

pub struct Protocol {
    channels: HashMap<PID, Sender<Vec<Query>>>,
    receiver: Receiver<HashMap<PID, Vec<Query>>>,
}

impl Protocol {
    pub fn new<const N: usize>(participants: [Participant; N]) -> Self {
        let (sender, receiver) = mpsc::channel();

        let channels = participants
            .into_iter()
            .fold(HashMap::new(), |mut map, participant| {
                // [NOTE]
                let (tx, rx) = mpsc::channel();
                map.insert(participant.id, tx);

                // [NOTE]
                let tx = sender.clone();
                thread::spawn(move || Protocol::work(participant, rx, tx));

                return map;
            });

        Self {
            channels: channels,
            receiver: receiver,
        }
    }

    pub fn seed(&self, participant: &Participant) {
        let queries = participant
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
                path: Vec::new(),
                source: node,
                target: node,
                token: Ciphertext::default(),
            })
            .collect();

        return self.dispatch([(participant.id, queries)].into());
    }

    fn dispatch(&self, jobs: HashMap<PID, Vec<Query>>) {
        for (id, queries) in jobs {
            self.channels
                .get(id)
                .expect("Participant must be known to dispatch job")
                .send(queries)
                .expect("Channel must be open to dispatch job");
        }
    }

    pub fn run(self) {
        let mut pending = 1;

        while pending > 0 {
            let jobs = self.receiver.recv().expect("");

            pending += jobs.len();
            self.dispatch(jobs);

            pending -= 1;
        }

        drop(self.channels);
        drop(self.receiver);
    }

    fn work(
        mut participant: Participant,
        receiver: Receiver<Vec<Query>>,
        sender: Sender<HashMap<PID, Vec<Query>>>,
    ) {
        let mut components: Vec<Component> = Vec::new();

        // [NOTE]
        for queries in receiver {
            debug_println!();
            debug_println!("--- PARTICIPANT {} START ---", participant.id);

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

            debug_println!("--- PARTICIPANT {} END ---", participant.id);

            // [NOTE]
            components.extend(complete);
            sender.send(queries).expect("");
        }

        // [NOTE]
        components.retain(|c| {
            let mut seen = HashSet::new();
            return c.iter().all(|x| seen.insert(x));
        });

        println!();
        debug_println!("Components: {components:?}");
        debug_println!("Count: {}", components.len());
    }
}
