use crate::core::address::P2PKHAddress;
use crate::crypto::public_key::PublicKeyAlgorithm;
use crate::crypto::vrf::{keygen};

#[derive(Clone)]
pub struct NodeKeyChain {
	/// Used for receiving the reward. (P2PKHAddress, Signing_key, Verifying_key)
	pub wallet_key_pair: (P2PKHAddress, Vec<u8>, Vec<u8>),
	/// Used for the block leader lottery. (Proving_key/Signing_key, Verifying_key)
	pub vrf_key_pair: ([u8; 32], [u8; 32]),
}
impl NodeKeyChain {
	pub fn random() -> Self {
		let (vrf_sk, vrf_pk) = keygen();
		Self {
			wallet_key_pair: P2PKHAddress::random(),
			vrf_key_pair: (vrf_sk.to_bytes(), vrf_pk.to_bytes())
		}
	}
}