use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode, Url};

use crate::network::{config, standard};
use crate::network::models::{BlockchainInfo, PairUp};
use crate::network::standard::{standard_deserialize, standard_serialize};

pub struct Sender;

impl Sender {
	pub async fn get_blockchain_info(client: &Client, mut peer: Url) -> Result<BlockchainInfo, Box<impl std::error::Error>> {
		peer.set_path(config::GET_BLOCKCHAIN_INFO_URL);
		return match client.get(peer)
			.send().await {
			Ok(response) => {
				if let Ok(bytes) = response.bytes().await {
					let data = bytes.to_vec();
					if let Ok(info) = standard_deserialize::<BlockchainInfo>(data.as_slice()) {
						return Ok(info);
					}
				}
				Err(Box::new(io::Error::new(ErrorKind::InvalidData, "Invalid data")))
			}
			Err(err) => {
				Err(Box::new(io::Error::new(ErrorKind::NotConnected, err.to_string())))
			}
		}
	}
	pub async fn send_bytes(client: &Client, url: Url, bytes: Vec<u8>) -> Result<String, String> {
		let send = client.post(url.clone())
			.body(bytes)
			.header(reqwest::header::CONTENT_TYPE, standard::DATA_TYPE) // Set the content type
			.send();

		if let Ok(result) = tokio::time::timeout(Duration::from_millis(500), send).await {
			match result {
				Ok(response) => {
					let txt = String::from_utf8(response.bytes().await.unwrap().to_vec()).unwrap();
					log::info!("Response: {:?}", txt);
					Ok(txt)
				}
				Err(err) => {
					Err(err.to_string())
				}
			}
		} else {
			Err("Timeout".to_string())
		}
	}
	pub async fn pair_up_with(client: &Client, peer: Url, msg: PairUp) -> reqwest::Result<bool> {
		let mut url = peer;
		url.set_path(config::PAIR_UP_URL);
		let response = client.post(url)
			.body(standard_serialize(&msg).expect("Unable to deserialize"))
			.header(reqwest::header::CONTENT_TYPE, standard::DATA_TYPE) // Set the content type
			.send().await?;
		Ok(response.status() == StatusCode::OK)
	}
	pub async fn get_peers(client: &Client, peer: Url) -> anyhow::Result<HashSet<String>> {
		let mut url = peer;
		url.set_path(config::GET_PEERS_URL);
		let response = client.get(url)
			.header(reqwest::header::CONTENT_TYPE, standard::DATA_TYPE) // Set the content type
			.send().await?;
		match response.bytes().await.map(|x|x.to_vec()) {
			Ok(data) => {
				match standard_deserialize::<HashSet<String>>(data.as_slice()) {
					Ok(info) => {
						Ok(info)
					}
					Err(e) => {
						Err(e)
					}
				}
			}
			Err(e) => {
				Err(e.into())
			}
		}
	}
}