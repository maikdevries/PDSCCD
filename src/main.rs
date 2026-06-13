use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, sync::Arc, time::Duration};

use pcd::private::{
    core::{Graph, Location, Node, PID, Participant},
    crypto::Crypto,
    protocol::Protocol,
};

struct Parameters {
    edges: usize,
    iterations: usize,
    length: usize,
    nodes: usize,
    participants: usize,
}

fn main() {
    let crypto = Arc::new(Crypto::new());
    let parameters = Parameters {
        edges: 8,
        iterations: 5,
        length: 4,
        nodes: 1,
        participants: 8,
    };

    let participants = [
        Participant::new(
            "A",
            Graph::new([
                Node::new(1, Location::Internal, [2, 3, 4, 5, 6, 7, 8]),
                Node::new(2, Location::External("B"), [1]),
                Node::new(3, Location::External("C"), [1]),
                Node::new(4, Location::External("D"), [1]),
                Node::new(5, Location::External("E"), [1]),
                Node::new(6, Location::External("F"), [1]),
                Node::new(7, Location::External("G"), [1]),
                Node::new(8, Location::External("H"), [1]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "B",
            Graph::new([
                Node::new(1, Location::External("A"), [2]),
                Node::new(2, Location::Internal, [1, 3, 4, 5, 6, 7, 8]),
                Node::new(3, Location::External("C"), [2]),
                Node::new(4, Location::External("D"), [2]),
                Node::new(5, Location::External("E"), [2]),
                Node::new(6, Location::External("F"), [2]),
                Node::new(7, Location::External("G"), [2]),
                Node::new(8, Location::External("H"), [2]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "C",
            Graph::new([
                Node::new(1, Location::External("A"), [3]),
                Node::new(2, Location::External("B"), [3]),
                Node::new(3, Location::Internal, [1, 2, 4, 5, 6, 7, 8]),
                Node::new(4, Location::External("D"), [3]),
                Node::new(5, Location::External("E"), [3]),
                Node::new(6, Location::External("F"), [3]),
                Node::new(7, Location::External("G"), [3]),
                Node::new(8, Location::External("H"), [3]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "D",
            Graph::new([
                Node::new(1, Location::External("A"), [4]),
                Node::new(2, Location::External("B"), [4]),
                Node::new(3, Location::External("C"), [4]),
                Node::new(4, Location::Internal, [1, 2, 3, 5, 6, 7, 8]),
                Node::new(5, Location::External("E"), [4]),
                Node::new(6, Location::External("F"), [4]),
                Node::new(7, Location::External("G"), [4]),
                Node::new(8, Location::External("H"), [4]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "E",
            Graph::new([
                Node::new(1, Location::External("A"), [5]),
                Node::new(2, Location::External("B"), [5]),
                Node::new(3, Location::External("C"), [5]),
                Node::new(4, Location::External("D"), [5]),
                Node::new(5, Location::Internal, [1, 2, 3, 4, 6, 7, 8]),
                Node::new(6, Location::External("F"), [5]),
                Node::new(7, Location::External("G"), [5]),
                Node::new(8, Location::External("H"), [5]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "F",
            Graph::new([
                Node::new(1, Location::External("A"), [6]),
                Node::new(2, Location::External("B"), [6]),
                Node::new(3, Location::External("C"), [6]),
                Node::new(4, Location::External("D"), [6]),
                Node::new(5, Location::External("E"), [6]),
                Node::new(6, Location::Internal, [1, 2, 3, 4, 5, 7, 8]),
                Node::new(7, Location::External("G"), [6]),
                Node::new(8, Location::External("H"), [6]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "G",
            Graph::new([
                Node::new(1, Location::External("A"), [7]),
                Node::new(2, Location::External("B"), [7]),
                Node::new(3, Location::External("C"), [7]),
                Node::new(4, Location::External("D"), [7]),
                Node::new(5, Location::External("E"), [7]),
                Node::new(6, Location::External("F"), [7]),
                Node::new(7, Location::Internal, [1, 2, 3, 4, 5, 6, 8]),
                Node::new(8, Location::External("H"), [7]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
        Participant::new(
            "H",
            Graph::new([
                Node::new(1, Location::External("A"), [8]),
                Node::new(2, Location::External("B"), [8]),
                Node::new(3, Location::External("C"), [8]),
                Node::new(4, Location::External("D"), [8]),
                Node::new(5, Location::External("E"), [8]),
                Node::new(6, Location::External("F"), [8]),
                Node::new(7, Location::External("G"), [8]),
                Node::new(8, Location::Internal, [1, 2, 3, 4, 5, 6, 7]),
            ]),
            crypto.clone(),
            parameters.length,
        ),
    ];

    let mut timings: HashMap<PID, Vec<HashMap<&str, Duration>>> = HashMap::new();

    for _ in 0..parameters.iterations {
        let (_, ts) = Protocol::new(participants.clone()).run("A");
        for (pid, ts) in ts {
            timings.entry(pid).or_default().push(ts);
        }
    }

    if let Err(_) = write_to_file(timings, parameters) {
        eprintln!();
        eprintln!("TIMINGS HAVE NOT BEEN WRITTEN TO DISK!");
    };
}

fn write_to_file(
    timings: HashMap<PID, Vec<HashMap<&str, Duration>>>,
    parameters: Parameters,
) -> std::io::Result<()> {
    let filename = format!(
        "{}P{}N{}E{}L.json",
        parameters.participants, parameters.nodes, parameters.edges, parameters.length
    );
    let file = File::create_new(Path::new("./benchmarks").join(filename))?;
    let writer = BufWriter::new(file);

    // [NOTE]
    let output: HashMap<PID, Vec<HashMap<&str, u128>>> = timings
        .into_iter()
        .map(|(pid, timings)| {
            (
                pid,
                timings
                    .into_iter()
                    .map(|x| {
                        x.into_iter()
                            .map(|(label, duration)| (label, duration.as_nanos()))
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect();

    serde_json::to_writer_pretty(writer, &output)?;
    return Ok(());
}
