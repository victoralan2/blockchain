use json::object;

use crate::network::routes::p2p::URL_REGEX;

pub enum ErrorType {
	WrongVersion(u32, u32), // first is the version that the other node had and second is the expected version
	InvalidTransaction(String),
	InvalidBlock(String),
	InvalidUrl,
}
impl ToString for ErrorType {
	fn to_string(&self) -> String {
		match self {
			Self::WrongVersion(request_version, expected_version) => {
				let json = object! {
					error: "WrongVersion",
					message: "Distinct protocol version",
					request_version: *request_version,
					expected_version: *expected_version
				};
				json.to_string()
			},
			Self::InvalidTransaction(context) => {
				let json = object! {
					error: "InvalidTransaction",
					message: "The transaction is invalid in the current context",
					context: context.to_string()
				};
				json.to_string()
			},
			ErrorType::InvalidBlock(context) => {
				let json = object! {
					error: "InvalidBlock",
					message: "The block is invalid in the current context",
					context: context.to_string()
				};
				json.to_string()
			}
			ErrorType::InvalidUrl => {
				let json = object! {
					error: "InvalidUrl",
					message: "The given url is invalid. Must match the URL_REGEX",
					valid_url_regex: URL_REGEX,
				};
				json.to_string()
			}
		}
	}
}