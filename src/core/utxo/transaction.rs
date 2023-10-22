use std::cmp::min;
use pqcrypto_dilithium::dilithium5::{SecretKey};
use serde::{Deserialize, Serialize};
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::Hashable;
use crate::core::utxo::{Input, Output};
use crate::crypto::public_key::Dilithium;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
	pub fn sign_inputs(&mut self, sk: &SecretKey) -> pqcrypto_traits::Result<()>{
		for input in self.input_list.iter_mut() {
			let hash = input.calculate_hash();
			let signature = Dilithium::sign_dilithium(&sk, &hash);
			input.signature = signature;

		}
		Ok(())
	}
	pub fn verify_inputs(&self, blockchain: &BlockChain) -> bool { // TODO
		for input in self.input_list {
			let utxo = blockchain.get_utxo(&input.prev_txid);
			if let Some(utxo) = utxo {
				let index = utxo.output_index;
				let pk = Dilithium::pkey_from_bytes(&input.public_key);
				if let Ok(pk) = pk {
					let signature_content = Dilithium::open_dilithium(&pk, &input.signature);
					if let Some(signature_content) = signature_content {
						if signature_content == input.calculate_hash() {
							continue
						}
					}
				}
			}
			return false
		}
		true
	}

	pub fn is_valid_heuristic(&self) -> bool {
		let is_signature_valid = self.verify_inputs();
		let are_outputs_valid = self.verify_outputs();
		is_signature_valid
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