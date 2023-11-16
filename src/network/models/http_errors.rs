use serde::{Deserialize, Serialize};


pub enum ErrorType {
	WrongVersion(u32, u32), // first is the version that the other node had and second is the expected version // TODO: Maybe make the state static so it can be looked up.

}
#[derive(Clone, Deserialize, Serialize)]
pub struct ErrorMessage {
	pub(crate) error: String,
	pub(crate) message: String,
}
impl ErrorType {
	pub fn to_message(&self) -> String {
		match self {
			ErrorType::WrongVersion(version, expected_version) => {
				let msg = ErrorMessage {
					error: "WrongVersion".to_string(),
					message: format!("Expected protocol version {}", expected_version),
				};
				serde_json::to_string_pretty(&msg).unwrap()
			},
		}
	}
}