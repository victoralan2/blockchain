use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use rand::thread_rng;
use rand_core::RngCore;

use crate::core::{Hashable, is_smaller};
use crate::core::address::P2PKHAddress;
use crate::core::block::Block;
use crate::network::models::NewBlock;
use crate::network::node::Node;

#[async_trait]
pub trait Miner {
	async fn mine_one(&mut self, miner: P2PKHAddress) -> Option<Block> ;
	async fn mine(&mut self, miner: P2PKHAddress);
}

#[async_trait]
impl Miner for Node {
	async fn mine_one(&mut self, miner: P2PKHAddress) -> Option<Block> {
		let blockchain = self.blockchain.read().await;
		let starting_height = blockchain.get_height();
		let config = blockchain.configuration;
		let target_value = config.target_value;
		let transaction_slice = blockchain.mempool.iter().take(1024).cloned().collect(); // TODO: Change this number to be depending on tx size
		let last_block_header = blockchain.get_last_block().header.clone();

		let mut new_block = Block::new(last_block_header.height,
									   transaction_slice,
									   SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
									   last_block_header.hash, miner, blockchain.configuration);



		new_block.header.nonce = thread_rng().next_u64();
		new_block.update_hash();

		let mut i = 0;
		const CHECK_RATE: u32 = 256;

		loop {
			new_block.header.nonce += 1;

			if !is_smaller(&new_block.calculate_hash(), &target_value) {
				return Some(new_block);
			} else {
				new_block.header.hash = new_block.calculate_hash();
			}

			if i >= CHECK_RATE {
				new_block.header.time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

				let blockchain = self.blockchain.read().await;

				if self.should_mine.load(Ordering::Relaxed) || blockchain.get_height() != starting_height || !new_block.is_valid(&blockchain, blockchain.get_height()) {
					return None;
				}

				i=0;
			}

			i+=1;
		}
	}

	async fn mine(&mut self, miner: P2PKHAddress) {
		println!("Trying to mine block...");
		while self.should_mine.load(Ordering::Relaxed) {
			if let Some(block) = Self::mine_one(self, miner).await {
				println!("NEW BLOCK MINED!"); // TODO: Alert the user

				let peers = self.peers.read().await.clone();
				self.blockchain.write().await.add_block(&block);
				let version = self.version;

				let msg = NewBlock {
					version,
					block,
				};
				Self::broadcast_block(peers, &msg).await;
			}
		}
	}
}

