use std::cmp::{min, Ordering};

use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::Hashable;
use crate::crypto::hash::hash;
use crate::crypto::public_key::Dilithium;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, Serialize, Deserialize)]
pub enum BlockData {
	TX(Transaction),
	Data(Data)
}
impl BlockData {
	pub fn is_valid_heuristic(&self, confg: &BlockChainConfig) -> bool {
		match self {
			BlockData::TX(tx) => {tx.is_valid_heuristic()}
			BlockData::Data(data) => {data.is_valid_heuristic(confg)}
		}
	}
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		match self {
			BlockData::TX(tx) => {tx.is_valid(blockchain)}
			BlockData::Data(data) => {data.is_valid(blockchain)}
		}
	}
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, Serialize, Deserialize)]
pub struct Transaction {
	pub nonce: u64,
	pub time: u64,
	pub hash: [u8; 32],
	pub sender_address: P2PKHAddress,
	pub sender_public_key: Vec<u8>,
	pub recipient_address: P2PKHAddress,
	pub amount: u64,
	pub signature: Vec<u8>,
}
impl Transaction {
	pub fn new(time: u64, nonce: u64, sender_address: &P2PKHAddress, sender_public_key: &PublicKey, recipient_address: &P2PKHAddress, amount: u64, signature: Vec<u8>) -> Self {
		let mut tx = Transaction {
			nonce,
			time,
			hash: [0u8; 32],
			sender_address: sender_address.clone(),
			sender_public_key: Dilithium::serialize_pkey(sender_public_key),
			recipient_address: recipient_address.clone(),
			amount,
			signature
		};
		tx.update_hash();
		tx
	}
	pub fn new_unsigned(time: u64, nonce: u64, sender: &P2PKHAddress, sender_public_key: &PublicKey, recipient_address: &P2PKHAddress, amount: u64) -> Self {
		Self::new(time, nonce,sender, sender_public_key, recipient_address, amount, vec![])
	}
	/// This function signs the transaction with the given key.
	/// It also __updates the hash__ of the transaction as a **side effect**
	pub fn sign(&mut self, sk: &SecretKey) -> pqcrypto_traits::Result<()>{
		self.update_hash();
		let hash = &self.hash;

		let signature = Dilithium::sign_dilithium(&sk, hash);
		self.signature = signature;
		Ok(())
	}

	pub fn validate_hash(&mut self) -> bool{
		self.calculate_hash() == self.hash
	}
	pub fn verify_signature(&self) -> bool {
		let pk = Dilithium::pkey_from_bytes(&self.sender_public_key);
		if let Ok(pk) = pk {
			if hash(&self.sender_public_key) != self.sender_address.address {
				return false;
			}
			let signature_content = Dilithium::open_dilithium(&pk, &self.signature);
			if let Some(signature_content) = signature_content {
				return signature_content == self.hash
			}
		}
		false
	}
	pub fn is_valid_heuristic(&self) -> bool {
		let is_signature_valid = self.verify_signature();
		let is_hash_valid = self.calculate_hash() == self.hash;
		is_signature_valid && is_hash_valid && (self.amount != 0)
	}
	/// Checks if the transaction's signature is valid, if the hash is valid and if the sender can afford to send this transaction
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		self.is_valid_heuristic() && self.is_affordable(blockchain)
	}
	pub fn is_affordable(&self, blockchain: &BlockChain) -> bool{
		let fee = self.calculate_fee(&blockchain.configuration);
		let amount = self.amount;
		let sender_balance = blockchain.get_balance(&self.sender_address);
		sender_balance >= fee + amount
	}
	pub fn calculate_fee(&self, config: &BlockChainConfig) -> u64 {
		let fee = (config.transaction_fee_multiplier * self.amount as f64).floor() as u64;
		let max_fee = config.max_transaction_fee;
		min(fee, max_fee)
	}
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
	pub time: u64,
	pub hash: [u8; 32],
	pub creator: P2PKHAddress,
	pub creator_public_key: Vec<u8>,
	pub signature: Vec<u8>,
	pub data: Vec<u8>,
}
impl Data {
	pub fn validate_hash(&mut self) -> bool{
		self.calculate_hash() == self.hash
	}

	pub fn verify_signature(&self) -> bool {
		let pk = Dilithium::pkey_from_bytes(&self.creator_public_key);
		if let Ok(pk) = pk {
			if hash(&self.creator_public_key) != self.creator.address {
				return false;
			}
			let signature_content = Dilithium::open_dilithium(&pk, &self.signature);
			if let Some(signature_content) = signature_content {
				return signature_content == self.hash
			}
		}
		false
	}
	pub fn calculate_fee(&self, config: &BlockChainConfig) -> u64 {
		(config.data_fee_multiplier * self.data.len() as f64).floor() as u64
	}
	pub fn is_valid_heuristic(&self, config: &BlockChainConfig) -> bool {
		self.verify_signature() && self.data.len() < config.max_data_size
	}
	/// Checks if the data's signature is valid, if the hash is valid and if the sender can afford to publish this data because of fees
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		self.is_valid_heuristic(&blockchain.configuration) && self.is_affordable(blockchain)
	}
	pub fn is_affordable(&self, blockchain: &BlockChain) -> bool{
		let fee = self.calculate_fee(&blockchain.configuration);
		let sender_balance = blockchain.get_balance(&self.creator);
		sender_balance >= fee
	}
}