use crate::primitives::Bytes32;
use blst::{min_pk as blst_core, BLST_ERROR};
use sha2::{digest::FixedOutput, Digest, Sha256};
use ssz_rs::prelude::*;
use std::fmt;
use thiserror::Error;

const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

pub fn hash(data: &[u8]) -> Bytes32 {
    let mut result = Bytes32::default();
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize_into(result.0.as_mut_slice().into());
    result
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Zero sized input")]
    ZeroSizedInput,
    #[error("randomness failure: {0}")]
    Randomness(#[from] rand::Error),
    #[error("blst error: {0}")]
    BLST(#[from] BLSTError),
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct BLSTError(String);

impl From<BLST_ERROR> for BLSTError {
    fn from(err: BLST_ERROR) -> Self {
        let inner = match err {
            BLST_ERROR::BLST_SUCCESS => unreachable!("do not create a BLSTError from a sucess"),
            BLST_ERROR::BLST_BAD_ENCODING => "bad encoding",
            BLST_ERROR::BLST_POINT_NOT_ON_CURVE => "point not on curve",
            BLST_ERROR::BLST_POINT_NOT_IN_GROUP => "point not in group",
            BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => "aggregation type mismatch",
            BLST_ERROR::BLST_VERIFY_FAIL => "verification failed",
            BLST_ERROR::BLST_PK_IS_INFINITY => "public key is infinity",
            BLST_ERROR::BLST_BAD_SCALAR => "bad scalar input",
        };
        Self(inner.to_string())
    }
}

#[derive(Clone, Default)]
pub struct SecretKey(blst_core::SecretKey);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for i in self.0.to_bytes() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl SecretKey {
    // https://docs.rs/rand/latest/rand/trait.Rng.html#generic-usage
    pub fn random<R: rand::Rng>(rng: &mut R) -> Result<Self, Error> {
        let mut ikm = [0u8; 32];
        rng.try_fill_bytes(&mut ikm)?;
        Self::key_gen(&ikm)
    }

    pub fn key_gen(ikm: &[u8]) -> Result<Self, Error> {
        let sk = blst_core::SecretKey::key_gen(ikm, &[]).map_err(BLSTError::from)?;
        Ok(SecretKey(sk))
    }

    pub fn from_bytes(encoding: &[u8]) -> Result<Self, Error> {
        let inner = blst_core::SecretKey::from_bytes(encoding).map_err(BLSTError::from)?;
        Ok(Self(inner))
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        let pk = self.0.sk_to_pk();
        PublicKey(pk)
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.0.sign(msg, BLS_DST, &[]))
    }
}

fn verify_signature(public_key: &PublicKey, msg: &[u8], sig: &Signature) -> bool {
    let pk = &public_key.0;
    let avg = &[];
    let res = sig.0.verify(true, msg, BLS_DST, avg, pk, true);
    res == BLST_ERROR::BLST_SUCCESS
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct PublicKey(blst_core::PublicKey);

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for i in self.0.to_bytes() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl PublicKey {
    pub fn verify_signature(&self, msg: &[u8], sig: &Signature) -> bool {
        verify_signature(self, msg, sig)
    }

    pub fn validate(&self) -> bool {
        self.0.validate().is_ok()
    }

    pub fn from_bytes(encoding: &[u8]) -> Result<Self, Error> {
        let inner = blst_core::PublicKey::from_bytes(encoding).map_err(BLSTError::from)?;
        Ok(Self(inner))
    }

    pub fn as_bytes(&self) -> [u8; 48] {
        self.0.to_bytes()
    }
}

impl Sized for PublicKey {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        48
    }
}

impl Serialize for PublicKey {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        let start = buffer.len();
        buffer.extend_from_slice(&self.as_bytes());
        let encoded_length = buffer.len() - start;
        debug_assert_eq!(encoded_length, Self::size_hint());
        Ok(encoded_length)
    }
}

impl Deserialize for PublicKey {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let inner = blst_core::PublicKey::deserialize(encoding)
            .map_err(|_| DeserializeError::InvalidInput)?;
        Ok(Self(inner))
    }
}

impl Merkleized for PublicKey {
    fn hash_tree_root(&self, context: &MerkleizationContext) -> Result<Node, MerkleizationError> {
        let mut buffer = vec![];
        self.serialize(&mut buffer)?;
        pack_bytes(&mut buffer);
        merkleize(&buffer, None, context)
    }
}

impl SimpleSerialize for PublicKey {
    fn is_composite_type() -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Signature(blst_core::Signature);

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for i in self.0.to_bytes() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl Default for Signature {
    fn default() -> Self {
        let mut encoding = vec![0u8; Self::size_hint()];
        // set top two bits, particularity of the `BLS12-381` encoding
        encoding[0] = 192;
        Self(
            blst_core::Signature::from_bytes(&encoding)
                .expect("default is a well-formed signature"),
        )
    }
}

impl Signature {
    pub fn verify(&self, pk: &PublicKey, msg: &[u8]) -> bool {
        verify_signature(pk, msg, self)
    }

    pub fn from_bytes(encoding: &[u8]) -> Result<Self, Error> {
        let sig = blst_core::Signature::from_bytes(encoding).map_err(BLSTError::from)?;
        Ok(Self(sig))
    }

    pub fn as_bytes(&self) -> [u8; 96] {
        self.0.to_bytes()
    }
}

impl Sized for Signature {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        96
    }
}

impl Serialize for Signature {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        let start = buffer.len();
        buffer.extend_from_slice(&self.as_bytes());
        let encoded_length = buffer.len() - start;
        debug_assert!(encoded_length == Self::size_hint());
        Ok(encoded_length)
    }
}

impl Deserialize for Signature {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let inner = blst_core::Signature::deserialize(encoding)
            .map_err(|_| DeserializeError::InvalidInput)?;
        Ok(Self(inner))
    }
}

impl Merkleized for Signature {
    fn hash_tree_root(&self, context: &MerkleizationContext) -> Result<Node, MerkleizationError> {
        let mut buffer = vec![];
        self.serialize(&mut buffer)?;
        pack_bytes(&mut buffer);
        merkleize(&buffer, None, context)
    }
}

impl SimpleSerialize for Signature {
    fn is_composite_type() -> bool {
        false
    }
}

pub fn aggregate(signatures: &[Signature]) -> Result<Signature, Error> {
    if signatures.is_empty() {
        return Err(Error::ZeroSizedInput);
    }
    let vs: Vec<&blst_core::Signature> = signatures.iter().map(|s| &s.0).collect();

    blst_core::AggregateSignature::aggregate(&vs, true)
        .map(|s| Signature(s.to_signature()))
        .map_err(|e| BLSTError::from(e).into())
}

pub fn aggregate_verify(pks: &[PublicKey], msgs: &[&[u8]], signature: Signature) -> bool {
    let v: Vec<&blst_core::PublicKey> = pks.iter().map(|pk| &pk.0).collect();
    let res = signature.0.aggregate_verify(true, msgs, BLS_DST, &v, true);
    res == BLST_ERROR::BLST_SUCCESS
}

pub fn fast_aggregate_verify(pks: &[PublicKey], msg: &[u8], signature: Signature) -> bool {
    let v: Vec<&blst_core::PublicKey> = pks.iter().map(|pk| &pk.0).collect();
    let res = signature.0.fast_aggregate_verify(true, msg, BLS_DST, &v);
    res == BLST_ERROR::BLST_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{aggregate, aggregate_verify, fast_aggregate_verify};
    use rand::prelude::*;

    #[test]
    fn signature() {
        let mut rng = thread_rng();
        let sk = SecretKey::random(&mut rng).unwrap();
        let pk = sk.public_key();
        let msg = "message";
        let sig = sk.sign(msg.as_ref());

        assert!(sig.verify(&pk, msg.as_ref()));

        let pk = sk.public_key();
        assert!(pk.verify_signature(msg.as_ref(), &sig));
    }

    #[test]
    #[should_panic(expected = "bad encoding")]
    fn test_signature_from_null_bytes() {
        let b = [0u8; 0];
        Signature::from_bytes(&b).expect("can make a signature");
    }

    #[test]
    fn test_signature_from_good_bytes() {
        let b = [
            0xab, 0xb0, 0x12, 0x4c, 0x75, 0x74, 0xf2, 0x81, 0xa2, 0x93, 0xf4, 0x18, 0x5c, 0xad,
            0x3c, 0xb2, 0x26, 0x81, 0xd5, 0x20, 0x91, 0x7c, 0xe4, 0x66, 0x65, 0x24, 0x3e, 0xac,
            0xb0, 0x51, 0x00, 0x0d, 0x8b, 0xac, 0xf7, 0x5e, 0x14, 0x51, 0x87, 0x0c, 0xa6, 0xb3,
            0xb9, 0xe6, 0xc9, 0xd4, 0x1a, 0x7b, 0x02, 0xea, 0xd2, 0x68, 0x5a, 0x84, 0x18, 0x8a,
            0x4f, 0xaf, 0xd3, 0x82, 0x5d, 0xaf, 0x6a, 0x98, 0x96, 0x25, 0xd7, 0x19, 0xcc, 0xd2,
            0xd8, 0x3a, 0x40, 0x10, 0x1f, 0x4a, 0x45, 0x3f, 0xca, 0x62, 0x87, 0x8c, 0x89, 0x0e,
            0xca, 0x62, 0x23, 0x63, 0xf9, 0xdd, 0xb8, 0xf3, 0x67, 0xa9, 0x1e, 0x84,
        ];
        Signature::from_bytes(&b).expect("can make a signature");
    }

    #[test]
    #[should_panic(expected = "bad encoding")]
    fn secret_key_is_null() {
        let ikm = [0u8; 0];
        SecretKey::key_gen(&ikm).expect("can make a secret key");
    }

    #[test]
    #[should_panic(expected = "bad encoding")]
    fn secret_key_len_31() {
        let ikm = [1u8; 31];
        SecretKey::key_gen(&ikm).expect("can make a secret key");
    }

    #[test]
    fn valid_public_key() {
        let mut rng = thread_rng();
        let pk = SecretKey::random(&mut rng).unwrap().public_key();
        let valid = pk.validate();
        assert!(valid)
    }

    #[test]
    fn test_aggregate_verify() {
        let n = 20;
        let mut rng = thread_rng();
        let sks: Vec<_> = (0..n)
            .map(|_| SecretKey::random(&mut rng).unwrap())
            .collect();
        let pks: Vec<_> = sks.iter().map(|sk| sk.public_key()).collect();
        let msgs: Vec<Vec<u8>> = (0..n)
            .map(|_| (0..64).map(|_| rand::thread_rng().gen()).collect())
            .collect();

        let signatures: Vec<_> = msgs
            .iter()
            .zip(&sks)
            .map(|(msg, sk)| sk.sign(msg.as_ref()))
            .collect();

        let msgs = msgs.iter().map(|r| &r[..]).collect::<Vec<_>>();

        let sig = aggregate(signatures.as_ref()).unwrap();
        let v = aggregate_verify(pks.as_slice(), msgs.as_ref(), sig);

        assert!(v);
    }

    #[test]
    fn test_fast_aggregated_verify() {
        let n = 20;
        let mut rng = thread_rng();
        let sks: Vec<_> = (0..n)
            .map(|_| SecretKey::random(&mut rng).unwrap())
            .collect();
        let pks: Vec<_> = sks.iter().map(|sk| sk.public_key()).collect();
        let msg = "message";

        let signatures: Vec<_> = sks.iter().map(|sk| sk.sign(msg.as_ref())).collect();

        let sig = aggregate(signatures.as_slice()).unwrap();
        let v = fast_aggregate_verify(pks.as_slice(), msg.as_ref(), sig);

        assert!(v);
    }

    #[test]
    fn test_can_make_default_signature() {
        let signature = Signature::default();
        dbg!(signature);
    }

    #[test]
    fn test_can_sign() {
        let secret_key_hex = "40094c5c6c378857eac09b8ec64c87182f58700c056a8b371ad0eb0a5b983d50";
        let secret_key_bytes = hex::decode(secret_key_hex).expect("is hex");
        let secret_key = SecretKey::from_bytes(&secret_key_bytes).unwrap();

        let signature_hex = "a01e49276730e4752eef31b0570c8707de501398dac70dd144438cd1bd05fb9b9bb3e1a9ceef0a68cc08904362cafa3f1005e5b699a41847fff6f5552260468846de5bdbf94a9aedeb29bc6cdb2c1d34922d9e9af4c0593a69ae978a90b5aba6";
        let signature_bytes = hex::decode(signature_hex).expect("can decode hex");
        let expected_signature = Signature::from_bytes(&signature_bytes).unwrap();

        let message = b"blst is such a blast";
        let signature = secret_key.sign(message);
        assert_eq!(expected_signature, signature);
    }

    #[test]
    fn test_roundtrip_bytes() {
        let mut rng = thread_rng();
        let secret_key = SecretKey::random(&mut rng).unwrap();
        let public_key = secret_key.public_key();
        let msg = b"blst is such a blast";
        let signature = secret_key.sign(msg);

        let secret_key_bytes = secret_key.as_bytes();
        let publicy_key_bytes = public_key.as_bytes();
        let signature_bytes = signature.as_bytes();

        let _ = SecretKey::from_bytes(&secret_key_bytes).unwrap();

        let recovered_public_key = PublicKey::from_bytes(&publicy_key_bytes).unwrap();
        assert_eq!(public_key, recovered_public_key);
        let recovered_signature = Signature::from_bytes(&signature_bytes).unwrap();
        assert_eq!(signature, recovered_signature);

        assert!(signature.verify(&public_key, msg));
    }
}
