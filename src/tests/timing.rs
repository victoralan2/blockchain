use std::hint::black_box;
use std::thread::sleep;
use std::time::{Duration, Instant};
use rsntp::{AsyncSntpClient, SntpClient, SynchroniztationError};
use crate::network::node::{Node, STARTING_SLOT_SECOND};
use crate::network::timing;
use crate::network::timing::sync_to_slot;

#[tokio::test]
pub async fn timing_test() {
	tokio::spawn(async move {
		let mut ntp = AsyncSntpClient::new();


		ntp.set_timeout(Duration::from_secs_f32(1.0));
		sync_to_slot(&ntp, 100).await;
		let start = Instant::now();
		let mut slot = timing::get_accurate_slot(&ntp, 100).await.unwrap();
		spin_sleep::sleep(Duration::from_secs_f32(0.1) - start.elapsed());

		for _ in 0..1000 {
			let start = Instant::now();
			log::info!("Current slot from A: {}", slot);
			slot+=1;
			spin_sleep::sleep(Duration::from_secs_f32(0.1) - start.elapsed());
		}
	});
	spin_sleep::sleep(Duration::from_secs_f32(2.32));

	tokio::spawn(async move {
		let mut ntp = AsyncSntpClient::new();


		ntp.set_timeout(Duration::from_secs_f32(1.0));
		sync_to_slot(&ntp, 100).await;
		let start = Instant::now();
		let mut slot = timing::get_accurate_slot(&ntp, 100).await.unwrap();
		spin_sleep::sleep(Duration::from_secs_f32(0.1) - start.elapsed());

		for _ in 0..1000 {
			let start = Instant::now();
			log::info!("Current slot from B: {}", slot);
			slot+=1;
			spin_sleep::sleep(Duration::from_secs_f32(0.1) - start.elapsed());
		}
	});
	tokio::signal::ctrl_c().await.ok();}

