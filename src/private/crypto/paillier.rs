use crypto_bigint::{
    RandomMod, U1024, U2048, U4096,
    modular::{FixedMontyForm, FixedMontyParams},
};
use crypto_primes::{Flavor, random_prime};

pub type Ciphertext = U4096;
pub type Plaintext = U2048;

struct Private {
    lambda: U2048,
    mu: U2048,
}

struct Public {
    modulus: U2048,
    squared: U4096,
}

pub struct Paillier {
    private: Private,
    public: Public,
}

impl Paillier {
    pub fn new() -> Self {
        let p = random_prime::<U1024, _>(&mut rand::rng(), Flavor::Any, 1024);
        let q = random_prime::<U1024, _>(&mut rand::rng(), Flavor::Any, 1024);

        assert_ne!(p, q, "Generated primes must not be equal");

        // [NOTE]
        let n = p.concatenating_mul(&q);
        let phi = (p - U1024::ONE).concatenating_mul(&(q - U1024::ONE));

        Self {
            private: Private {
                lambda: phi,
                mu: phi
                    .invert_odd_mod_vartime(&n.to_odd().unwrap())
                    .expect("Modular inverse must exist"),
            },
            public: Public {
                modulus: n,
                squared: n.concatenating_square(),
            },
        }
    }

    pub fn encrypt(&self, plaintext: &Plaintext) -> Ciphertext {
        // [NOTE]
        let cipher = self
            .public
            .modulus
            .concatenating_mul(plaintext)
            .add_mod(&U4096::ONE, &self.public.squared.to_nz_vartime().unwrap());

        // [NOTE]
        let r = U2048::random_mod_vartime(
            &mut rand::rng(),
            &self.public.modulus.to_nz_vartime().unwrap(),
        );

        // [TODO] Greatest common divisor of r and modulus must be 1
        assert_ne!(r, U2048::ZERO, "Randomness must not be zero");

        // [NOTE]
        let randomness = FixedMontyForm::new(
            &r.resize(),
            &FixedMontyParams::new_vartime(self.public.squared.to_odd().unwrap()),
        )
        .pow_vartime(&self.public.modulus)
        .retrieve();

        // [NOTE]
        return cipher.mul_mod_vartime(&randomness, &self.public.squared.to_nz_vartime().unwrap());
    }

    pub fn decrypt(&self, cipher: &Ciphertext) -> Plaintext {
        // [NOTE]
        let x = FixedMontyForm::new(
            cipher,
            &FixedMontyParams::new_vartime(self.public.squared.to_odd().unwrap()),
        )
        .pow_vartime(&self.private.lambda)
        .retrieve();

        // [NOTE]
        let L = (x - U4096::ONE) / self.public.modulus;

        // [NOTE]
        return (L * self.private.mu) % self.public.modulus;
    }
}
