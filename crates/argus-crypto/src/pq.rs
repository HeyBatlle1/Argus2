//! Post-quantum cryptography primitives
//!
//! NIST-standardized algorithms, implemented via the `pqcrypto` family of crates:
//!   - ML-KEM / Kyber-768  (FIPS 203) — Key Encapsulation
//!   - ML-DSA / Dilithium-3 (FIPS 204) — Digital Signatures
//!
//! Both modules return owned `Vec<u8>` so callers stay key-type agnostic.
//! Key material can be stored in `SecureVault` and retrieved for each operation.

pub mod kem {
    //! Key Encapsulation Mechanism — ML-KEM / Kyber-768

    use pqcrypto_kyber::kyber768;
    use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};

    pub struct KemKeypair {
        pub public_key: Vec<u8>,
        pub secret_key: Vec<u8>,
    }

    pub struct EncapsulatedKey {
        pub ciphertext: Vec<u8>,
        pub shared_secret: Vec<u8>,
    }

    /// Generate a fresh ML-KEM-768 keypair.
    pub fn generate_keypair() -> KemKeypair {
        let (pk, sk) = kyber768::keypair();
        KemKeypair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }

    /// Encapsulate: generate a shared secret and a ciphertext for the given public key.
    /// The sender keeps `shared_secret`; the `ciphertext` is sent to the recipient.
    pub fn encapsulate(public_key: &[u8]) -> Result<EncapsulatedKey, String> {
        let pk = kyber768::PublicKey::from_bytes(public_key)
            .map_err(|e| format!("ML-KEM: invalid public key — {}", e))?;
        let (ss, ct) = kyber768::encapsulate(&pk);
        Ok(EncapsulatedKey {
            ciphertext: ct.as_bytes().to_vec(),
            shared_secret: ss.as_bytes().to_vec(),
        })
    }

    /// Decapsulate: recover the shared secret from a ciphertext using the secret key.
    pub fn decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let sk = kyber768::SecretKey::from_bytes(secret_key)
            .map_err(|e| format!("ML-KEM: invalid secret key — {}", e))?;
        let ct = kyber768::Ciphertext::from_bytes(ciphertext)
            .map_err(|e| format!("ML-KEM: invalid ciphertext — {}", e))?;
        let ss = kyber768::decapsulate(&ct, &sk);
        Ok(ss.as_bytes().to_vec())
    }
}

pub mod sign {
    //! Digital Signatures — ML-DSA / Dilithium-3

    use pqcrypto_dilithium::dilithium3;
    use pqcrypto_traits::sign::{DetachedSignature, PublicKey, SecretKey};

    pub struct SigningKeypair {
        pub public_key: Vec<u8>,
        pub secret_key: Vec<u8>,
    }

    /// Generate a fresh ML-DSA-65 (Dilithium-3) keypair.
    pub fn generate_keypair() -> SigningKeypair {
        let (pk, sk) = dilithium3::keypair();
        SigningKeypair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }

    /// Sign a message; returns a detached signature (not prepended to message).
    pub fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
        let sk = dilithium3::SecretKey::from_bytes(secret_key)
            .map_err(|e| format!("ML-DSA: invalid secret key — {}", e))?;
        let sig = dilithium3::detached_sign(message, &sk);
        Ok(sig.as_bytes().to_vec())
    }

    /// Verify a detached signature. Returns `true` if valid.
    pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let pk = match dilithium3::PublicKey::from_bytes(public_key) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig = match dilithium3::DetachedSignature::from_bytes(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        dilithium3::verify_detached_signature(&sig, message, &pk).is_ok()
    }
}
