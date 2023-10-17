use std::fmt::format;
use pqcrypto_dilithium::dilithium5::SecretKey;
use crate::core::address::Address;
use crate::core::Hashable;
use crate::crypto::hash::hash;
use crate::crypto::public_key::Dilithium;

#[derive(Clone, Debug, PartialEq)]
pub struct CoinBaseTransaction {
	pub receiver: Address,
	pub amount: u64,
}

impl CoinBaseTransaction {
	pub fn new(receiver: Address, amount: u64) -> Self {
		CoinBaseTransaction { receiver, amount }
	}
	pub fn null() -> Self {
		CoinBaseTransaction { receiver: Address::null(), amount: 0 }
	}
}
#[derive(Clone, Debug, PartialEq)]
pub enum BlockData {
	TX(Transaction),
	Data(Data)
}
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
	pub time: u64,
	pub hash: [u8; 32],
	pub sender_address: Address,
	pub recipient_address: Address,
	pub amount: u64,
	pub signature: Vec<u8>,
}
impl Transaction {
	pub fn new(time: u64, sender_address: &Address, recipient_address: &Address, amount: u64, signature: Vec<u8>) -> Self {
		let mut tx = Transaction {
			time,
			hash: [0u8; 32],
			sender_address: sender_address.clone(),
			recipient_address: recipient_address.clone(),
			amount,
			signature
		};
		tx.update_hash();
		tx
	}
	pub fn new_unsigned(time: u64, sender: &Address, recipient_address: &Address, amount: u64) -> Self {
		Self::new(time, sender, recipient_address, amount, vec![])
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
		let pk = self.sender_address.public_key;
		let signature_content = Dilithium::open_dilithium(&pk, &self.signature);
		if let Some(signature_content) = signature_content {
			signature_content == self.hash
		} else {
			false
		}
	}
	pub fn is_valid(&self) -> bool {
		let is_signature_valid = self.verify_signature();
		println!("{}", is_signature_valid);
		is_signature_valid
	}
}
#[derive(Clone, Debug, PartialEq)]
pub struct Data {
	pub time: u64,
	pub hash: Vec<u8>,
	pub creator: Option<Address>,
	pub signature: Option<Vec<u8>>,
	pub data: Vec<u8>,
}
impl Data {
	pub fn is_anonymous(&self) -> bool {
		return self.creator.is_some();
	}
	pub fn update_hash(&mut self) {
		todo!()
	}
	pub fn calculate_hash(&self) -> Vec<u8>{
		todo!()
	}
	pub fn validate_hash(&mut self) -> bool{
		todo!()
	}
	pub fn verify_signature(&self) -> bool {
		if self.is_anonymous() {
			return true;
		}
		todo!()
	}
	pub fn is_valid(&self) -> bool {
		// TODO: SIZE IS BELOW SOME LIMIT
		todo!()
	}
}