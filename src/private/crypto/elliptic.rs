use curve25519_dalek::{RistrettoPoint, Scalar};

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
    public: RistrettoPoint,
    secret: Scalar,
}

impl Elliptic {
    pub fn new() -> Self {
        let secret = Scalar::random(&mut rand::rng());

        Self {
            public: RistrettoPoint::mul_base(&secret),
            secret: secret,
        }
    }

    pub fn encode(message: u128) -> Plaintext {
        return RistrettoPoint::mul_base(&Scalar::from(message));
    }

    pub fn encrypt(&self, plaintext: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: plaintext + r * self.public,
            randomness: RistrettoPoint::mul_base(&r),
        }
    }

    fn decrypt(&self, cipher: &Ciphertext) -> Plaintext {
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
        blinds: Vec<Plaintext>,
    ) -> (Vec<Unsealed>, Vec<Plaintext>) {
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
        let blinds = blinds.into_iter().map(|blind| blind * r).collect();

        return (unsealed, blinds);
    }
}
