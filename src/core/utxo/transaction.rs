use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::{Input, Output};
use crate::crypto::public_key::{PublicKeyAlgorithm, PublicKeyError};

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
	pub id: [u8; 32],
	pub extra_entropy: u16,
	pub input_list: Vec<Input>,
	pub output_list: Vec<Output>,
}

impl Transaction {
	pub fn create_transaction(inputs: Vec<Input>, outputs: Vec<Output>, extra_entropy: u16) -> Self {
		let mut s = Self {
			id: [0u8; 32],
			extra_entropy: 0,
			input_list: vec![],
			output_list: vec![],
		};
		s.update_hash();
		s
	}
	pub fn sign_inputs(&mut self, sk: &[u8]) -> Result<(), PublicKeyError> {
		for input in self.input_list.iter_mut() {
			let hash = input.calculate_hash();
			let signature = PublicKeyAlgorithm::sign(&sk, &hash)?;
			input.signature = signature;
		}
		Ok(())
	}
	pub fn verify_input_signatures(&self) -> bool {
		for input in &self.input_list {
			if !input.verify_signature() {
				return false;
			}
		}
		true
	}
	pub fn validate_inputs(&self, blockchain: &BlockChain) -> bool {
		 for input in &self.input_list {
			 if !input.validate(blockchain) {
				 return false;
			 }
		 }
		true
	}
	pub fn do_sum(&self, blockchain: &BlockChain) -> bool {
		let mut spent = 0;
		for output in &self.output_list {
			spent += output.amount;
		}
		let mut budget = 0;
		for input in & self.input_list {
			if let Some(utxos) = blockchain.get_utxo_list(&input.prev_txid) {
				if let Some(utxo) = utxos.get(input.output_index) {
					budget += utxo.amount;
				}
			}
		}
		budget == spent
	}
	pub fn is_valid_heuristic(&self) -> bool {
		let is_tx_size_valid = self.input_list.len() < 128 && self.output_list.len() < 128;
		let are_inputs_unique = self.are_inputs_unique();
		let are_signatures_valid = self.verify_input_signatures();
		are_signatures_valid && are_inputs_unique && is_tx_size_valid
	}
	pub fn are_inputs_unique(&self) -> bool {
		let mut output_indexes = HashSet::new();
		for input in &self.input_list {
			if !output_indexes.insert(input.calculate_hash()) {
				return false;
			}
		}
		true
	}
	/// Checks if the transaction's signature is valid, if the hash is valid and if the sender can afford to send this transaction
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		// TODO: CHECK FOR THE FEE OUTPUT OR SMT
		self.is_valid_heuristic() && self.do_sum(blockchain) && self.validate_inputs(blockchain)
	}
}