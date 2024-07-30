use serde::{Deserialize, Serialize};
use crate::core::block::Block;

use crate::core::utxo::UTXO;

// TODO: Just a reminder, for reversing blocks, remember reversing the coinbase tx too! Or however it works with Cardano / Ouroboros
#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct UndoTransaction {
	pub original_tx_id: [u8; 32],
	/// The UTxOs that the transaction removed to the thing
	pub removed_utxos: Vec<([u8; 32], UTXO)>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UndoBlock {
	pub height: usize,
	pub original_hash: [u8; 32],
	pub undo_transactions: Vec<UndoTransaction>,
}
impl UndoBlock {
	pub fn genesis() -> Self {
		Self {
			height: 0,
			original_hash: Block::genesis().header.hash,
			undo_transactions: vec![],
		}
	}
}