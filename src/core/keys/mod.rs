use crate::core::address::P2PKHAddress;

#[derive(Clone)]
pub struct NodeKeyChain {
	/// Used for receiving the reward. (P2PKHAddress, Signing_key, Verifying_key)
	pub wallet_key_pair: (P2PKHAddress, Vec<u8>, Vec<u8>),
}
impl NodeKeyChain {
	pub fn random() -> Self {
		Self {
			wallet_key_pair: P2PKHAddress::random(),
		}
	}
}