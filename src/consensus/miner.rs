use std::cmp::min;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;
use log::{info, log};
use num_bigint::{BigInt, Sign};
use rand_core::{RngCore, SeedableRng};
use sha3::digest::consts::P2;
use tokio::sync::{Mutex, RwLock, Semaphore};
use crate::core::address::P2PKHAddress;
use crate::core::block::{Block, BlockHeader};
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::hash;
use crate::crypto::hash::merkle::calculate_merkle_root;


pub struct Miner;

impl Miner {
	pub async fn start_mining(arc_blockchain: Arc<RwLock<BlockChain>>, reward_address: P2PKHAddress, should_mine: Arc<AtomicBool>) -> Block {
		// TODO: Test
		const HASHES_PER_UPDATE: u64 = 100000;
		
		let parameters = arc_blockchain.read().await.parameters;
		let max_tx_size = parameters.network_parameters.max_tx_size;
		
		let max_block_body_size = parameters.network_parameters.max_block_body_size;
		let mut transactions = arc_blockchain.read().await.mempool.get_map().iter().take(max_block_body_size / max_tx_size).cloned().collect();
		let mut target_difficulty = parameters.network_parameters.proof_of_work_difficulty;
		
		let mut rng = rand_xorshift::XorShiftRng::from_entropy();

		let mut i = 0;
		let mut header = BlockHeader {
			hash: [0u8; 32],
			nonce: 0,
			height: arc_blockchain.read().await.get_height() + 1,
			previous_hash: arc_blockchain.read().await.get_last_block().header.hash,
			merkle_root: calculate_merkle_tree(&transactions),
			miner_address: reward_address,
		};
		loop {
			header.nonce = rng.next_u64();

			let hash = calculate_hash(header);
			if hash < target_difficulty {
				header.hash = hash;
				return Block{ header, transactions };
			}

			if i == HASHES_PER_UPDATE {
				// Sleep until should mine
				while !should_mine.load(Ordering::Relaxed) {
					tokio::time::sleep(Duration::from_secs(1)).await;
				}


				header.height = arc_blockchain.read().await.get_height() + 1;
				target_difficulty = parameters.network_parameters.proof_of_work_difficulty;
				transactions = arc_blockchain.read().await.mempool.get_map().iter().take(max_block_body_size / max_tx_size).cloned().collect();
				header.previous_hash = arc_blockchain.read().await.get_last_block().header.hash;
				header.merkle_root = calculate_merkle_tree(&transactions);

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
fn calculate_hash(block_header: BlockHeader) -> [u8; 32]{
	let header = &block_header;
	let str = format!("{}.{}.{}.{}.{}", hex::encode(header.previous_hash), hex::encode(block_header.merkle_root), header.nonce, header.height, header.miner_address);
	hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
}


