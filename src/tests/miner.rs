use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crate::consensus::gen_difficulty;
use crate::consensus::miner::Miner;
use crate::core::address::P2PKHAddress;


#[tokio::test(flavor = "multi_thread")]
async fn miner_test(){
	let target_difficulty = gen_difficulty(0.00001);
	let miner = Arc::new(Mutex::new(Miner::new(vec![], 1, [0u8; 32], P2PKHAddress::random().0, target_difficulty)));
	let miner_clone = miner.clone();

	// tokio::spawn(async move {
	// 	Miner::start_mining(miner_clone).await;
	// });
	// 
	// sleep(Duration::from_secs(5));
	// let mut miner_lock = miner.lock().unwrap();
	// miner_lock.height = 10;
}