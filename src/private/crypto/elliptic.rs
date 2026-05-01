use curve25519_dalek::{RistrettoPoint, Scalar, constants::RISTRETTO_BASEPOINT_POINT as G};
use std::collections::HashMap;

// ---

impl std::fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Ciphertext");
    }
}

// ---

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Ciphertext {
    message: RistrettoPoint,
    randomness: RistrettoPoint,
}

impl std::ops::Mul<Scalar> for Ciphertext {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self {
            message: self.message * rhs,
            randomness: self.randomness * rhs,
        }
    }
}

pub type Plaintext = RistrettoPoint;

pub struct Sealed {
    pub nonce: u128,
    pub token: Ciphertext,
}

pub struct Unsealed {
    pub nonce: u128,
    pub token: Plaintext,
}

pub struct Elliptic {
    lookup: HashMap<[u8; 32], u32>,
    public: RistrettoPoint,
    secret: Scalar,
}

impl Elliptic {
    // [NOTE]
    const B: u32 = u32::MAX.isqrt() + 1;

    pub fn new() -> Self {
        let secret = Scalar::random(&mut rand::rng());

        Self {
            lookup: Self::generate_lookup(),
            public: RistrettoPoint::mul_base(&secret),
            secret: secret,
        }
    }

    fn generate_lookup() -> HashMap<[u8; 32], u32> {
        let mut lookup = HashMap::new();
        let mut baby = RistrettoPoint::default();

        // [NOTE]
        for i in 0..Self::B {
            lookup.insert(baby.compress().to_bytes(), i);
            baby += G;
        }

        return lookup;
    }

    pub fn encode<T>(message: T) -> Plaintext
    where
        T: Into<Scalar>,
    {
        return RistrettoPoint::mul_base(&message.into());
    }

    pub fn encrypt(&self, plaintext: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: plaintext + r * self.public,
            randomness: RistrettoPoint::mul_base(&r),
        }
    }

    pub fn decrypt(&self, cipher: &Ciphertext) -> Plaintext {
        return cipher.message - self.secret * cipher.randomness;
    }

    pub fn rerandomise(&self, cipher: &Ciphertext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: cipher.message + r * self.public,
            randomness: cipher.randomness + RistrettoPoint::mul_base(&r),
        }
    }

    pub fn unseal(
        &self,
        seals: Vec<Sealed>,
        blinds: Vec<Unsealed>,
    ) -> (Vec<Unsealed>, Vec<Unsealed>) {
        let r = Scalar::random(&mut rand::rng());

        // [NOTE]
        let unsealed = seals
            .into_iter()
            .map(|seal| Unsealed {
                nonce: seal.nonce,
                token: self.decrypt(&seal.token) * r,
            })
            .collect();

        // [NOTE]
        let blinds = blinds
            .into_iter()
            .map(|blind| Unsealed {
                nonce: blind.nonce,
                token: blind.token * r,
            })
            .collect();

        return (unsealed, blinds);
    }

    pub fn recover(&self, point: &Plaintext) -> Option<u32> {
        let mut giant = *point;

        // [NOTE]
        for i in 0..=Self::B {
            if let Some(j) = self.lookup.get(&giant.compress().to_bytes()) {
                return Some(i * Self::B + j);
            }

            giant -= RistrettoPoint::mul_base(&Scalar::from(Self::B));
        }

        return None;
    }
}
