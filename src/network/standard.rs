use std::error::Error;
use actix_bincode::BincodeSerde;
use serde::Serialize;

pub type DefaultExtractor<T> = BincodeSerde<T>;
pub fn serialize<T>(object: &T) -> Result<Vec<u8>, Box<dyn Error>>
	where T: Serialize{
	match bincode::serialize(object) {
		Ok(data) => {
			Ok(data)
		}
		Err(err) => {
			Err(Box::new(err))
		}
	}
}
