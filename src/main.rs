use pcd::private::{
    core::{Graph, Location, Node},
    protocol::Protocol,
};

fn main() {
    let participants = [
        (
            "A",
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        (
            "B",
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        (
            "C",
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let start = std::time::Instant::now();
    Protocol::new(participants).run("A");

    println!();
    println!("Duration: {:?}", std::time::Instant::now() - start);
}
