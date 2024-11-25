use std::cmp::min;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::consensus::gen_difficulty;
use crate::consensus::miner::Miner;
use crate::core::address::P2PKHAddress;


#[tokio::test(flavor = "multi_thread")]
async fn miner_test(){
	// TODO
	// let target_difficulty = gen_difficulty(1000);
	// let miner = Arc::new(Mutex::new(Miner::new(vec![], 1, [0u8; 32], P2PKHAddress::random().0, target_difficulty)));
	// let should_mine = Arc::new(AtomicBool::new(true));
	// let should_mine_clone = should_mine.clone();
	// let t = tokio::spawn(async move {
	// 	Miner::start_mining(miner_clone, should_mine_clone).await
	// });
	// sleep(Duration::from_secs(1)).await;
	// assert!(t.is_finished());
	// 
	// let b = t.await.unwrap();
	// assert!(b.header.verify_proof_of_work(target_difficulty));
	// 
	// let miner_clone = miner.clone();
	// let should_mine_clone = should_mine.clone();
	// 
	// let target_difficulty = gen_difficulty(1000000);
	// miner.lock().await.target_difficulty = target_difficulty;
	// 
	// const TEST_HEIGHT: usize = 100;
	// tokio::spawn(async move {
	// 	let c = should_mine_clone.clone();
	// 	while c.load(Ordering::Relaxed) {
	// 		let c1 = miner_clone.clone();
	// 		let c2 = should_mine_clone.clone();
	// 
	// 		let b = Miner::start_mining(c1, c2).await;
	// 		assert_eq!(b.header.height, TEST_HEIGHT)
	// 	}
	// });
	// miner.lock().await.height = TEST_HEIGHT;
}