use std::fmt::Formatter;
use std::str::FromStr;

use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PeerUrl(Url);
impl PeerUrl {
	pub fn new(url: Url) -> Self {
		Self(url)
	}
	pub fn from_url(url: Url) -> Self {
		Self(url)
	}
	pub fn to_url(&self) -> Url {
		self.0.clone()
	}
}
impl Serialize for PeerUrl {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(self.0.as_str())
	}
}
pub struct UrlVisitor;
impl<'de> Visitor<'de> for UrlVisitor {
	type Value = PeerUrl;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		write!(formatter, "a url formatted string")
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
		if let Ok(url) = Url::from_str(&v) {
			Ok(PeerUrl(url))
		} else {
			Err(Error::custom("Unable to parse string to url"))
		}
	}
}
impl<'de> Deserialize<'de> for PeerUrl {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		deserializer.deserialize_str(UrlVisitor)
	}
}