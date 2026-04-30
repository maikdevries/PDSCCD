use crate::private::crypto::elliptic::*;

pub mod elliptic;
// mod paillier;
// mod threshold;

pub struct Crypto {
    pub elliptic: Elliptic,
    // pub paillier: Paillier,
}

impl Crypto {
    pub fn new() -> Self {
        Self {
            elliptic: Elliptic::new(),
            // paillier: Paillier::new(),
        }
    }
}
