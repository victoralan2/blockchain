use crate::blockchain::address::Address;

#[derive(Clone)]
pub struct CoinBaseTransaction {
	pub receiver: Address,
	pub amount: f64,
}

#[derive(Clone)]
pub struct Transaction {
	time: u64,
	hash: Vec<u8>,
	sender_address: Vec<u8>,
	recipient_address: Vec<u8>,
	amount: f64,
	signature: Vec<u8>,
}


impl CoinBaseTransaction {
	pub fn new(receiver: Address, amount: f64) -> Self {
		CoinBaseTransaction { receiver, amount }
	}
	pub fn null() -> Self {
		CoinBaseTransaction { receiver: Address::null(), amount: 0f64 }
	}
}