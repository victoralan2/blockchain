use std::time::Duration;
use rsntp::{AsyncSntpClient, SynchroniztationError};
use crate::network::node::STARTING_SLOT_SECOND;


/// Syncs to the next slot
pub async fn sync_to_slot(ntp_client: &AsyncSntpClient, slot_time_in_millis: u64) {
	const ONE_MICROSECOND: u64 = 1000000;
	// Sync to the slot time
	let offset_in_micros = (ntp_client.synchronize("time.google.com").await
		.expect("Unable to sync with NTP server")
		.datetime()
		.unix_timestamp()
		.expect("Unable to sync with NTP server") * slot_time_in_millis as u32 / 1000).subsec_micros();
	spin_sleep::sleep(Duration::from_micros(ONE_MICROSECOND - offset_in_micros as u64));
}
pub async fn get_accurate_slot(ntp_client: &AsyncSntpClient, slot_time_in_millis: u64) -> Result<u64, SynchroniztationError> {
	let slot_time = ntp_client.synchronize("time.google.com").await?
		.datetime()
		.unix_timestamp()
		.expect("Time went backwards") - Duration::from_secs(STARTING_SLOT_SECOND);
	Ok(slot_time.as_millis() as u64 / slot_time_in_millis)
}