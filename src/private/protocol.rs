use std::collections::{HashMap, VecDeque};

use crate::private::{
    core::{Graph, Location, PID, Participant, Query, Request, Response},
    crypto::{Ciphertext, Partial, Plaintext, Threshold},
    tarjan::Component,
};

// ---

#[cfg(debug_assertions)]
macro_rules! debug_println {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_println {
    ($( $args:expr ),*) => {
        ()
    };
}

impl std::fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ciphertext")
    }
}

impl std::fmt::Debug for Plaintext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plaintext")
    }
}

impl std::fmt::Debug for Partial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Partial")
    }
}

// ---

enum Message {
    Query(Query),
    Request(Request),
    Response(Response),
}

pub struct Protocol {
    participants: HashMap<PID, Participant>,
    queue: VecDeque<(PID, Vec<Message>)>,
}

impl Protocol {
    pub fn new<const N: usize>(participants: [(PID, Graph); N]) -> Self {
        Self {
            participants: participants
                .into_iter()
                .zip(Threshold::setup::<3, N>())
                .map(|((id, graph), share)| (id, Participant::new(id, share, graph)))
                .collect(),
            queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self, initiator: PID, ttl: usize) -> HashMap<PID, Vec<Component>> {
        let participant = self
            .participants
            .get(initiator)
            .expect("Participant must have known ID");

        self.queue
            .push_back((initiator, Protocol::prepare(participant)));

        return self.process(ttl);
    }

    fn prepare(participant: &Participant) -> Vec<Message> {
        return participant
            .graph
            .nodes
            .values()
            .filter(|n| matches!(n.location, Location::External(_)) && n.neighbours.len() > 0)
            // [BUG] Creates unnecessary duplicate queries - some external nodes might share targets
            .flat_map(|external| {
                external.neighbours.iter().map(|node| {
                    Message::Query(Query {
                        from: "",
                        path: Vec::new(),
                        target: *node,
                        token: Ciphertext::default(),
                        ttl: 0,
                    })
                })
            })
            .collect();
    }

    fn process(&mut self, ttl: usize) -> HashMap<PID, Vec<Component>> {
        let mut components: HashMap<PID, Vec<Component>> = HashMap::new();

        while let Some((id, mut messages)) = self.queue.pop_front() {
            // [NOTE] Collect all consecutive requests for same participant into single batch
            while let Some((_, next)) = self.queue.pop_front_if(|(next, _)| *next == id) {
                messages.extend(next);
            }

            let participant = self
                .participants
                .get_mut(id)
                .expect("Participant must have known ID");

            let (queries, requests, responses) = Protocol::unwrap(messages);

            debug_println!();
            debug_println!("--- PARTICIPANT {id} START ---");

            // [NOTE]
            let (complete, incomplete) = participant.combine(responses);
            debug_println!("Complete: {complete:?}");
            debug_println!("Incomplete: {incomplete:?}");

            // [NOTE]
            let responses = participant.decrypt(requests);
            debug_println!("Responses: {responses:?}");

            // [NOTE]
            let (known, unknown) = participant.receive(queries);
            debug_println!("Known: {known:?}");
            debug_println!("Unknown: {unknown:?}");

            // [NOTE]
            let requests = participant.request(known);
            debug_println!("Requests: {requests:?}");

            // [NOTE]
            let detected = participant.detect(unknown.iter().map(|query| query.target).collect());
            debug_println!("Detected: {detected:?}");

            // [NOTE]
            let registered =
                participant.register(unknown.iter().map(|query| query.target).collect(), ttl);
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
            self.queue
                .extend(Protocol::wrap(queries, requests, responses));
        }

        return components;
    }

    fn unwrap(messages: Vec<Message>) -> (Vec<Query>, Vec<Request>, Vec<Response>) {
        let mut queries = Vec::new();
        let mut requests = Vec::new();
        let mut responses = Vec::new();

        for message in messages {
            match message {
                Message::Query(query) => queries.push(query),
                Message::Request(request) => requests.push(request),
                Message::Response(response) => responses.push(response),
            }
        }

        return (queries, requests, responses);
    }

    fn wrap(
        queries: HashMap<PID, Vec<Query>>,
        requests: HashMap<PID, Vec<Request>>,
        responses: HashMap<PID, Vec<Response>>,
    ) -> HashMap<PID, Vec<Message>> {
        let mut messages: HashMap<PID, Vec<Message>> = HashMap::new();

        for (k, v) in queries {
            messages
                .entry(k)
                .or_default()
                .extend(v.into_iter().map(Message::Query));
        }

        for (k, v) in requests {
            messages
                .entry(k)
                .or_default()
                .extend(v.into_iter().map(Message::Request));
        }

        for (k, v) in responses {
            messages
                .entry(k)
                .or_default()
                .extend(v.into_iter().map(Message::Response));
        }

        return messages;
    }
}
