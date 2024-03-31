use serde::{Deserialize, Serialize};

use crate::core::utxo::UTXO;

// TODO: Just a reminder, for reversing blocks, remember reversing the coinbase tx too! Or however it works with Cardano / Ouroboros

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct ReverseTransaction {
	pub original_tx_id: [u8; 32],
	/// The UTxOs that the transaction removed to the thing
	pub removed_utxos: Vec<UTXO>,
}

impl ReverseTransaction {
	pub fn create_reverse_transaction(original_transaction_id: [u8; 32], removed_utxos: Vec<UTXO>) -> Self {
		Self {
			original_tx_id: original_transaction_id,
			removed_utxos,
		}
	}
}