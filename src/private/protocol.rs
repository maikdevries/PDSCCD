use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Instant,
};

use crate::private::{
    core::{Location, Message, NID, PID, Participant},
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

#[derive(serde::Serialize)]
pub struct Resources {
    pub communication: HashMap<&'static str, u128>,
    pub messages: usize,
    pub space: HashMap<&'static str, u128>,
    pub time: HashMap<&'static str, u128>,
}

pub struct Protocol {
    participants: HashMap<PID, Participant>,
}

impl Protocol {
    pub fn new(participants: Vec<Participant>) -> Self {
        Self {
            participants: participants.into_iter().map(|p| (p.id, p)).collect(),
        }
    }

    pub fn run(
        self,
        initiator: PID,
    ) -> (
        HashMap<PID, HashMap<NID, Component>>,
        HashMap<PID, Resources>,
    ) {
        let participant = self
            .participants
            .get(initiator)
            .expect("Particpant must have known ID");

        // [NOTE]
        let jobs = Protocol::seed(participant);
        let mut pending = jobs.len();

        // [NOTE]
        let mut components = HashMap::new();
        let mut resources = HashMap::new();

        // [NOTE]
        thread::scope(|scope| {
            let mut channels = HashMap::new();
            let (sender, receiver) = mpsc::channel();

            // [NOTE]
            let (r_sender, r_receiver) = mpsc::channel();

            // [NOTE]
            for (id, participant) in self.participants {
                let (tx, rx) = mpsc::channel();
                channels.insert(id, tx);

                // [NOTE]
                let tx = sender.clone();
                let rtx = r_sender.clone();
                scope.spawn(move || Protocol::work(participant, rx, tx, rtx));
            }

            // [NOTE]
            sender.send(jobs).unwrap();

            // [NOTE]
            while pending > 0 {
                // [NOTE]
                let jobs = receiver.recv().unwrap();

                for (id, queries) in jobs {
                    channels[id].send(queries).unwrap();
                    pending += 1;
                }

                pending -= 1;
            }

            pending = channels.len();
            drop(channels);

            while pending > 0
                && let Ok((id, cs, rs)) = r_receiver.recv()
            {
                components.insert(id, cs);
                resources.insert(id, rs);

                pending -= 1;
            }
        });

        return (components, resources);
    }

    fn seed(participant: &Participant) -> HashMap<PID, Vec<Message>> {
        return [(
            participant.id,
            participant
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
                .map(|node| Message {
                    capacity: 0,
                    nodes: Vec::new(),
                    source: node,
                    target: node,
                    token: Ciphertext::default(),
                })
                .collect(),
        )]
        .into();
    }

    fn work(
        mut participant: Participant,
        receiver: Receiver<Vec<Message>>,
        sender: Sender<HashMap<PID, Vec<Message>>>,
        results: Sender<(PID, HashMap<NID, Component>, Resources)>,
    ) {
        let mut components: HashMap<NID, Component> = HashMap::new();

        let mut communication: HashMap<&str, u128> = HashMap::new();
        let mut space: HashMap<&str, u128> = HashMap::new();
        let mut time: HashMap<&str, u128> = HashMap::new();

        // [NOTE]
        macro_rules! time {
            ($label:literal, $expr:expr) => {{
                let t = Instant::now();
                let result = $expr;

                *time.entry($label).or_default() += t.elapsed().as_nanos();
                result
            }};
        }

        // [NOTE]
        let mut m = 0;
        let t = Instant::now();

        // [NOTE]
        for queries in receiver {
            debug_println!();
            debug_println!("--- PARTICIPANT {} START ---", participant.id);

            m += queries.len();
            *space.entry("receive").or_default() += std::mem::size_of_val(&queries) as u128;

            // [NOTE]
            let (known, unknown) = time!("receive", participant.receive(queries));
            debug_println!("[{}] - Known: {known:?}", participant.id);
            debug_println!("[{}] - Unknown: {unknown:?}", participant.id);

            // [NOTE]
            let (complete, incomplete, cs, ss) = time!("decrypt", participant.decrypt(known));
            debug_println!("[{}] - Complete: {complete:?}", participant.id);
            debug_println!("[{}] - Incomplete: {incomplete:?}", participant.id);

            *communication.entry("decrypt").or_default() += cs;
            *space.entry("decrypt").or_default() += ss;

            let targets = unknown.iter().map(|message| message.target).collect();

            // [NOTE]
            let (detected, ss) = time!("detect", participant.detect(&targets));
            debug_println!("[{}] - Detected: {detected:?}", participant.id);

            *space.entry("detect").or_default() += ss;

            // [NOTE]
            let (registered, ss) = time!("register", participant.register(targets));
            debug_println!("[{}] - Registered: {registered:?}", participant.id);

            *space.entry("register").or_default() += ss;

            // [NOTE]
            let queries = time!(
                "forward",
                participant.forward(
                    incomplete
                        .into_iter()
                        .chain(unknown)
                        .chain(registered)
                        .collect(),
                )
            );
            debug_println!("[{}] - Queries: {queries:?}", participant.id);
            *communication.entry("forward").or_default() += queries
                .values()
                .flatten()
                .map(|m| std::mem::size_of_val(m) as u128)
                .sum::<u128>();

            debug_println!("--- PARTICIPANT {} END ---", participant.id);

            // [NOTE]
            for (k, v) in complete {
                components.entry(k).or_default().extend(v);
            }

            // [NOTE]
            sender.send(queries).unwrap();
        }

        let t = t.elapsed();
        time.insert("total", t.as_nanos());

        debug_println!(
            "Participant {} detected {} components in {:?}",
            participant.id,
            components.len(),
            t
        );

        results
            .send((
                participant.id,
                components,
                Resources {
                    communication: communication,
                    messages: m,
                    space: space,
                    time: time,
                },
            ))
            .unwrap();
    }
}
