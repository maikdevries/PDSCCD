use crate::distributed::core::{Location, Participant, Query};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Protocol {
    participants: HashMap<&'static str, Participant>,
    queue: VecDeque<(&'static str, Vec<Query>)>,
}

impl Protocol {
    pub fn new(participants: HashMap<&'static str, Participant>) -> Self {
        Self {
            participants,
            queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self, initiator: &'static str) -> Vec<HashSet<usize>> {
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
            .filter(|n| matches!(n.location, Location::External(_)) && n.neighbours.len() > 0)
            .flat_map(|external| {
                let token = rand::random::<u128>();
                return external
                    .neighbours
                    .iter()
                    .map(move |&n| Query::new(n, token));
            })
            .collect();
    }

    fn process(&mut self) -> Vec<HashSet<usize>> {
        let mut results = Vec::new();

        while let Some((id, mut queries)) = self.queue.pop_front() {
            // [NOTE] Collect all consecutive 'requests' for same participant into single batch
            while let Some((_, other)) = self.queue.pop_front_if(|(other, _)| *other == id) {
                queries.extend(other);
            }

            let participant = self
                .participants
                .get_mut(id)
                .expect("Participant must have known ID");

            println!();
            println!("--- PARTICIPANT {id} START ---");

            let (resolved, unresolved) = participant.receive(queries);
            println!("Resolved: {resolved:?}");
            println!("Unresolved: {unresolved:?}");

            let (components, candidates) = Participant::compute(&participant.graph, unresolved);
            println!("Components: {components:?}");
            println!("Candidates: {candidates:?}");

            // [TODO] Do not send queries straight back to originating participant
            let queries = participant.send(candidates);
            println!("Queries: {queries:?}");

            println!("--- PARTICIPANT {id} END ---");
            self.queue.extend(queries);

            results.extend(resolved.into_iter().map(|q| q.nodes).chain(components));
        }

        results.dedup();
        return results;
    }
}
