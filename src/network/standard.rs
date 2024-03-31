use actix_web::web::Json;
use bincode::error::{DecodeError, EncodeError};
use serde::de::DeserializeOwned;
use serde::Serialize;

// const DATA_TYPE: &str = "application/octet-stream";
pub(crate) const DATA_TYPE: &str = "application/json"; // TODO CHANGE THIS AT "application/octet-stream" WHEN NOT TESTING

pub type StandardExtractor<T> = Json<T>; // TODO CHANGE THIS AT BincodeSerde WHEN NOT TESTING
// pub type StandardExtractor<T> = BincodeSerde<T>;

// pub fn standard_serialize<T>(object: &T) -> anyhow::Result<Vec<u8>> // TODO CHANGE THIS AT BincodeSerde WHEN NOT TESTING
// 	where T: Serialize{
// 	match serialize_bincode(object) {
// 		Ok(data) => {
// 			Ok(data)
// 		}
// 		Err(err) => {
// 			log::error!("Unable to serialize object. Error: {}", err);
// 			Err(err.into())
// 		}
// 	}
// }
// pub fn standard_deserialize<T>(object: &[u8]) -> anyhow::Result<T>
// 	where T: DeserializeOwned {
// 	Ok(deserialize_bincode(object)?.0)
// }
pub fn standard_serialize<T>(object: &T) -> anyhow::Result<Vec<u8>> // TODO CHANGE THIS AT BincodeSerde WHEN NOT TESTING
	where T: Serialize{
	Ok(serde_json::to_vec(object)?)
}
pub fn standard_deserialize<'a, T>(object: &'a [u8]) -> anyhow::Result<T>
	where T: serde::Deserialize<'a> {
	Ok(serde_json::from_slice::<T>(object)?)
}

pub fn deserialize_bincode<T>(object: &[u8]) -> Result<(T, usize), DecodeError>
	where T: DeserializeOwned {
	bincode::serde::decode_from_slice(object, bincode::config::standard())
}

pub fn serialize_bincode<T>(object: &T) -> Result<Vec<u8>, EncodeError>
	where T: Serialize {
	bincode::serde::encode_to_vec(object, bincode::config::standard())
}