use crate::private::crypto::elliptic::Elliptic;

pub mod elliptic;

mod threshold;

pub struct Crypto {
    pub elliptic: Elliptic,
}

impl Crypto {
    pub fn new() -> Self {
        Self {
            elliptic: Elliptic::new(),
        }
    }
}
