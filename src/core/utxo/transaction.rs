use pqcrypto_dilithium::dilithium5::{SecretKey};
use serde::{Deserialize, Serialize};
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::Hashable;
use crate::core::utxo::{Input, Output, UTXO};
use crate::crypto::public_key::Dilithium;

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
	pub id: [u8; 32],
	pub input_list: Vec<Input>,
	pub output_list: Vec<Output>,
	pub time: u64,
}

impl Transaction {
	// pub fn new(id: u64, input_list: Vec<Input>, output_list: Vec<Output>, time: u64) -> Self {
	// 	let mut tx = Transaction {
	// 		id,
	// 		input_list,
	// 		output_list,
	// 		time,
	// 		hash: [0u8; 32],
	// 	};
	// 	tx.update_hash();
	// 	tx
	// }
	// pub fn new_unsigned(id: u64, input_list: Vec<Input>, output_list: Vec<Output>, time: u64) -> Self {
	// 	Self::new(id, input_list, output_list, time)
	// }
	pub fn create_transaction(inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
		let mut s = Self {
			id: [0u8; 32],
			input_list: vec![],
			output_list: vec![],
			time: 0,
		};
		s.update_hash();
		s
	}
	pub fn sign_inputs(&mut self, sk: &SecretKey) -> pqcrypto_traits::Result<()> {
		for input in self.input_list.iter_mut() {
			let hash = input.calculate_hash();
			let signature = Dilithium::sign_dilithium(&sk, &hash);
			input.signature = signature;
		}
		Ok(())
	}
	pub fn verify_input_signatures(&self) -> bool {
		for input in &self.input_list {
			if !input.verify_signature(self) {
				return false;
			}
		}
		true
	}
	pub fn validate_inputs(&self, blockchain: &BlockChain) -> bool {
		 for input in &self.input_list {
			 if !input.validate(self, blockchain) {
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
		let are_signatures_valid = self.verify_input_signatures();
		are_signatures_valid
	}
	/// Checks if the transaction's signature is valid, if the hash is valid and if the sender can afford to send this transaction
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		self.is_valid_heuristic() && self.do_sum(blockchain) && self.validate_inputs(blockchain)
	}
	pub fn create_utxo_list(&self) -> Vec<UTXO>{
		let txid = self.id;
		let mut utxos = Vec::new();
		for (i, output) in self.output_list.iter().enumerate() {
			let utxo = UTXO {
				txid,
				output_index: i,
				amount: output.amount,
				recipient_address: output.address.clone(),
			};
			utxos.push(utxo);
		}
		utxos
	}
	pub fn calculate_fee(&self, config: &BlockChainConfig) -> u64 { // TODO: CALCULATE FEE
		// let fee = (config.transaction_fee_multiplier * self.output_list as f64).floor() as u64;
		// let max_fee = config.max_transaction_fee;
		// min(fee, max_fee)
		0
	}
}