use pcd::private::{
    core::{Graph, Location, Node},
    protocol::Protocol,
};

fn main() {
    let participants = [
        (
            "A",
            6,
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::External("B"), []),
            ]),
        ),
        (
            "B",
            6,
            Graph::new([
                Node::new(2, Location::External("A"), [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::External("C"), []),
            ]),
        ),
        (
            "C",
            6,
            Graph::new([
                Node::new(4, Location::External("B"), [5]),
                Node::new(5, Location::Internal, [0]),
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
            ]),
        ),
    ];

    let start = std::time::Instant::now();
    let components = Protocol::new(participants).run("A");

    println!();
    println!("Components: {components:?}");
    println!("Duration: {:?}", std::time::Instant::now() - start);
}
