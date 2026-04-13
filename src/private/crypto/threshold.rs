use curve25519_dalek::{RistrettoPoint, Scalar, constants::RISTRETTO_BASEPOINT_POINT as G};

use crate::private::crypto::elliptic::{Ciphertext, Plaintext};

struct Threshold {
    public: RistrettoPoint,
    share: Share,
}

impl Threshold {
    fn setup(threshold: usize, participants: usize) -> Vec<Self> {
        let coefficients: Vec<Scalar> = (0..threshold)
            .map(|_| Scalar::random(&mut rand::rng()))
            .collect();

        let public = coefficients[0] * G;

        return (1..=participants)
            .map(|i| Self {
                public: public,
                share: Share::new(&coefficients, i),
            })
            .collect();
    }

    fn encrypt(&self, message: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            randomness: r * G,
            message: message.point + r * self.public,
        }
    }

    fn partial(&self, cipher: &Ciphertext) -> Partial {
        Partial {
            index: self.share.index,
            value: self.share.value * cipher.randomness,
        }
    }

    fn decrypt(shares: &[Partial], cipher: Ciphertext) -> Plaintext {
        let indices: Vec<usize> = shares.iter().map(|share| share.index).collect();
        let mut secret = RistrettoPoint::default();

        for share in shares {
            let lambda = Partial::lagrange(&indices, share.index);
            secret += lambda * share.value;
        }

        Plaintext {
            point: cipher.message - secret,
        }
    }

    fn rerandomise(&self, cipher: &Ciphertext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            randomness: cipher.randomness + r * G,
            message: cipher.message + r * self.public,
        }
    }
}

struct Share {
    index: usize,
    value: Scalar,
}

impl Share {
    fn new(coefficients: &[Scalar], index: usize) -> Self {
        let x = Scalar::from(index as u128);

        let mut result = Scalar::ZERO;
        let mut power = Scalar::ONE;

        for coefficient in coefficients {
            result += coefficient * power;
            power *= x;
        }

        Share {
            index: index,
            value: result,
        }
    }
}

struct Partial {
    index: usize,
    value: RistrettoPoint,
}

impl Partial {
    fn lagrange(indices: &[usize], index: usize) -> Scalar {
        let x = Scalar::from(index as u128);

        let mut numerator = Scalar::ONE;
        let mut denominator = Scalar::ONE;

        for &i in indices {
            if i == index {
                continue;
            }

            let y = Scalar::from(i as u128);
            numerator *= -y;
            denominator *= x - y;
        }

        return numerator * denominator.invert();
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::IndexedRandom;

    use super::*;
    use crate::private::crypto::elliptic::Plaintext;

    #[test]
    fn encrypt_probabilistic() {
        let participants = Threshold::setup(3, 8);

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        assert_ne!(a, b);
    }

    #[test]
    fn decrypt() {
        let participants = Threshold::setup(3, 8);

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let cipher = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let shares: Vec<Partial> = participants
            .sample(&mut rand::rng(), 3)
            .map(|share| share.partial(&cipher))
            .collect();

        let plain = Threshold::decrypt(&shares, cipher);

        assert_eq!(plain, message);
    }

    #[test]
    fn decrypt_insufficient() {
        let participants = Threshold::setup(3, 8);

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let cipher = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let shares: Vec<Partial> = participants
            .sample(&mut rand::rng(), rand::random_range(1..3))
            .map(|share| share.partial(&cipher))
            .collect();

        let plain = Threshold::decrypt(&shares, cipher);

        assert_ne!(plain, message);
    }

    #[test]
    fn rerandomise_indistinguishable() {
        let participants = Threshold::setup(3, 8);

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .rerandomise(&a);

        assert_ne!(a, b);
    }

    #[test]
    fn rerandomise_preserving() {
        let participants = Threshold::setup(3, 8);

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .rerandomise(&a);

        let shares: Vec<Partial> = participants
            .sample(&mut rand::rng(), 3)
            .map(|share| share.partial(&b))
            .collect();

        let plain = Threshold::decrypt(&shares, b);

        assert_eq!(plain, message);
    }
}
