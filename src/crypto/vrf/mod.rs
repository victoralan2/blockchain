use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT as g;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
/// This file is a slightly edited copy of the ecvrf library.
/// The original library can be found at https://github.com/Silur/ECVRF
/// Credits to Silur, deuszex, HAOYUatHZ and Anfauglith for creating the library

use rand_core::OsRng;
use sha3::{Digest, Sha3_256 as SHA3, Sha3_512};

fn sha3(b: Vec<u8>) -> [u8; 32] {
	let mut hasher = SHA3::default();
	hasher.update(b);
	let r = hasher.finalize();
	let mut ret = [0u8; 32];
	for i in 0..r.len() {
		ret[i] = r[i];
	}
	ret
}

#[derive(Debug, PartialEq, Eq)]
/// Holds a secret key scalar for generating VRF proofs
pub struct VrfSk {
	s: Scalar
}

#[derive(Debug, PartialEq, Eq)]
/// Holds a public key g*sk as usual in ECC
pub struct VrfPk {
	p: RistrettoPoint
}

#[derive(Debug, PartialEq, Eq)]
/// The proof part of our VRF generation
pub struct VrfProof {
	gamma: RistrettoPoint,
	c: [u8; 32],
	s: Scalar,
}
impl Default for VrfSk {
	fn default() -> Self {
		VrfSk { s: Scalar::random(&mut OsRng) }
	}
}
impl VrfSk {
	pub fn to_bytes(&self) -> [u8; 32] {
		self.s.to_bytes()
	}
	pub fn from_bytes(b: &[u8; 32]) -> Result<VrfSk, &str> {
		let s = Scalar::from_canonical_bytes(*b);
		if s.is_none().unwrap_u8() == 1 { // Meaning that is none
			return Err("private key is not canonical");
		}
		Ok(VrfSk{ s: s.unwrap() })
	}
}

impl Clone for VrfSk {
	fn clone(&self) -> Self {
		VrfSk { s: self.s }
	}
}

impl VrfPk {
	pub fn new(sk: &VrfSk) -> VrfPk {
		VrfPk { p: g*sk.s }
	}
	pub fn to_bytes(&self) -> [u8; 32] {
		self.p.compress().to_bytes()
	}
	pub fn from_bytes(b: &[u8; 32]) -> Result<VrfPk, &str> {
		if let Ok(_p) = CompressedRistretto::from_slice(b) {
			let p = _p.decompress();
			if p.is_none() { return Err("public key is not canonical"); }

			return Ok(VrfPk{ p: p.unwrap() });
		}
		Err("Unable to convert from slice")
	}
}

impl Clone for VrfPk {
	fn clone(&self) -> Self {
		VrfPk { p: self.p }
	}
}

impl VrfProof {
	pub fn to_bytes(&self) -> [u8;96]{
		let mut ret = [0;96];
		let gamma = self.gamma.compress().to_bytes();
		let c = self.c;
		let s = self.s.to_bytes();
		ret[..32].copy_from_slice(&gamma[..32]);
    	ret[32..(32 + 32)].copy_from_slice(&c[..32]);
    	ret[64..(32 + 64)].copy_from_slice(&s[..32]);
		ret
	}
	pub fn from_bytes(input: &[u8;96]) -> Result<VrfProof, &str> {
		let mut gamma : [u8;32]=[0;32];
		let mut c : [u8;32]=[0;32];
		let mut s : [u8;32]=[0;32];
		gamma.copy_from_slice(&input[0..32]);
		c.copy_from_slice(&input[32..64]);
		s.copy_from_slice(&input[64..96]);

		let gamma_point = match CompressedRistretto::from_slice(&gamma) {
			Ok(gamma_point) => {gamma_point.decompress()}
			Err(_) => {
				return Err("Unable to convert from slice")
			}
		};
		let s_reduced = Scalar::from_canonical_bytes(s);
		if gamma_point.is_none() || s_reduced.is_none().unwrap_u8() == 1 { return Err("cannot deserialize vrf proof"); }
		Ok(VrfProof {
			gamma: gamma_point.unwrap(),
			c,
			s: s_reduced.unwrap()
		})
	}
}

impl Clone for VrfProof {
	fn clone(&self) -> Self {
		VrfProof {
			gamma: self.gamma,
			c: self.c,
			s: self.s,
		}
	}
}

fn serialize_point(p: RistrettoPoint) -> [u8; 32] {
	p.compress().to_bytes()
}

/// Generates a crypto-safe secret key using OsRng and the
/// corresponding public key into a tuple
pub fn keygen() -> (VrfSk, VrfPk) {
	let sk = VrfSk::default();
	let pk = VrfPk::new(&sk);
	(sk, pk)
}
/// The output of a VRF function is the VRF hash and the proof to verify
/// we generated this hash with the supplied key
pub fn prove(input: &[u8], privkey: &VrfSk) -> ([u8; 32], VrfProof) {
	let h = RistrettoPoint::hash_from_bytes::<Sha3_512>(input);
	let gamma = h * privkey.s;
	let k: Scalar = Scalar::random(&mut OsRng);
	let mut hasher = SHA3::default();
	hasher.update(serialize_point(g));
	hasher.update(serialize_point(h));
	hasher.update(serialize_point(g * privkey.s));
	hasher.update(serialize_point(h * privkey.s));
	hasher.update(serialize_point(g * k));
	hasher.update(serialize_point(h * k));
	let mut c = [0u8; 32];
	let hres = hasher.finalize();
	for i in 0..hres.len() {
		c[i] = hres[i];
	}
	let c_scalar = Scalar::from_bytes_mod_order(c);
	let s = k - c_scalar * privkey.s;
	let beta = sha3(serialize_point(gamma).to_vec());
	(beta, VrfProof { gamma, c, s })
}

pub fn verify(input: &[u8], pubkey: &VrfPk, output: &[u8; 32], proof: &VrfProof) -> bool {
	let c_scalar = Scalar::from_bytes_mod_order(proof.c);
	let u = pubkey.p * c_scalar + g * proof.s;
	let h = RistrettoPoint::hash_from_bytes::<Sha3_512>(input);
	let v = proof.gamma * c_scalar + h * proof.s;

	let mut hasher = SHA3::default();
	hasher.update(serialize_point(g));
	hasher.update(serialize_point(h));
	hasher.update(serialize_point(pubkey.p));
	hasher.update(serialize_point(proof.gamma));
	hasher.update(serialize_point(u));
	hasher.update(serialize_point(v));
	let mut local_c = [0_u8; 32];
	let hres = hasher.finalize();
	for i in 0..hres.len() {
		local_c[i] = hres[i];
	}
	sha3(serialize_point(proof.gamma).to_vec()) == *output && local_c == proof.c
}
#[cfg(test)]
mod tests {
	#[test]
	fn correct_proof() {
		let (privkey, pubkey) = super::keygen();
		let input = vec![0xde, 0xad, 0xbe, 0xef];
		let (output, proof) = super::prove(&input, &privkey);

		assert!(super::verify(&input, &pubkey, &output, &proof));

	}
	#[test]
	fn serialize() {
		let (privkey, pubkey) = super::keygen();
		let input = vec![0xde, 0xad, 0xbe, 0xef];
		let (_, proof) = super::prove(&input, &privkey);
		let sk_serialized = privkey.to_bytes();
		let pk_serialized = pubkey.to_bytes();
		let proof_serialized = proof.to_bytes();
		assert_eq!(super::VrfSk::from_bytes(&sk_serialized).unwrap(), privkey);
		assert_eq!(super::VrfPk::from_bytes(&pk_serialized).unwrap(), pubkey);
		assert_eq!(super::VrfProof::from_bytes(&proof_serialized).unwrap(), proof);
	}

	#[test]
	fn forgery() {
		let (privkey, pubkey) = super::keygen();
		let (forge_privkey, forge_pubkey) = super::keygen();
		let input = vec![0xde, 0xad, 0xbe, 0xef];
		let input_forged = vec![0xde, 0xad, 0xbe, 0xed];
		let (output, proof) = super::prove(&input, &privkey);
		let (forge_output, forge_proof) = super::prove(&input, &forge_privkey);
		let mut output_forged = output;
		output_forged[0] += 0x01;

		assert!(!super::verify(&input_forged, &pubkey, &output, &proof));
		assert!(!super::verify(&input, &pubkey, &output_forged, &proof));
		assert!(!super::verify(&input, &pubkey, &forge_output, &proof));
		assert!(!super::verify(&input, &pubkey, &output, &forge_proof));
		assert!(!super::verify(&input, &forge_pubkey, &output, &proof));
	}
}
