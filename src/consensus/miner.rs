use std::cmp::min;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;
use log::log;
use num_bigint::{BigInt, Sign};
use rand_core::{RngCore, SeedableRng};
use sha3::digest::consts::P2;
use tokio::sync::{Mutex, Semaphore};
use crate::core::address::P2PKHAddress;
use crate::core::block::{Block, BlockHeader};
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::hash;
use crate::crypto::hash::merkle::calculate_merkle_root;

#[derive(Clone)]
pub struct Miner {
	pub transactions: Vec<Transaction>,
	pub height: usize,
	pub last_hash: [u8; 32],
	pub reward_address: P2PKHAddress,
	pub target_difficulty: [u8; 32],
}

impl Miner {
	pub fn new(transactions: Vec<Transaction>, height: usize, last_hash: [u8; 32], reward_address: P2PKHAddress, target_difficulty: [u8; 32]) -> Self {
		Self {
			transactions,
			height,
			last_hash,
			reward_address,
			target_difficulty,
		}
	}

	pub async fn start_mining(arc_self: Arc<Mutex<Self>>, should_mine: Arc<AtomicBool>) -> Block {
		
		const HASHES_PER_UPDATE: u64 = 100000;
		
		let mut _self = (*arc_self.lock().await).clone();

		let mut height = _self.height;
		let mut nonce;
		let mut transactions = _self.transactions;
		let mut last_hash = _self.last_hash;
		let mut reward_address = _self.reward_address;
		let mut target_difficulty = _self.target_difficulty;
		let mut merkle_root = calculate_merkle_tree(&transactions);
		let mut rng = rand_xorshift::XorShiftRng::from_entropy();
		
		let mut i = 0;
		loop {
			nonce = rng.next_u64();

			let header = BlockHeader{
				hash: [0u8; 32],
				nonce,
				height,
				previous_hash: last_hash,
				merkle_root,
				miner_address: reward_address,
			};
			let hash = calculate_hash(header.clone(), merkle_root);
			if hash < target_difficulty {
				return Block{ header, transactions };
			}
			
			if i == HASHES_PER_UPDATE {
				// Sleep until should mine
				while !should_mine.load(Ordering::Relaxed) {
					tokio::time::sleep(Duration::from_secs(1)).await;
				}
				
				
				// Check if data has changed
				_self = (*arc_self.lock().await).clone();
				height = _self.height;
				transactions = _self.transactions;
				last_hash = _self.last_hash;
				reward_address = _self.reward_address;
				target_difficulty = _self.target_difficulty;
				merkle_root = calculate_merkle_tree(&transactions);

				i=0;
			}
			i+=1;
		}

	}
}

fn calculate_merkle_tree(txs: &Vec<Transaction>) -> [u8; 32]{
	let mut hashes: Vec<[u8; 32]> = Vec::new();
	for tx in txs {
		hashes.push(tx.calculate_hash());
	}
	calculate_merkle_root(hashes)
}
fn calculate_hash(block_header: BlockHeader, merkle_root: [u8; 32]) -> [u8; 32]{
	let header = &block_header;
	let str = format!("{}.{}.{}.{}.{}", hex::encode(header.previous_hash), hex::encode(merkle_root), header.nonce, header.height, header.miner_address);
	hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
}


