use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::io::{Bytes, ErrorKind};
use reqwest::{Client, Response, StatusCode, Url};
use serde_json::to_vec;
use crate::core::blockchain::BlockChain;
use crate::network::config;
use crate::network::models::{BlockchainInfo, GetHeaders, PairUp};
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
	pub async fn send_bytes(client: &Client, url: Url, bytes: Vec<u8>) -> reqwest::Result<Response> {
		client.post(url)
			.body(bytes)
			.header(reqwest::header::CONTENT_TYPE, "application/octet-stream") // Set the content type
			.send().await
	}
	pub async fn pair_up_with(client: &Client, peer: Url, msg: PairUp) -> reqwest::Result<bool> {
		let mut url = peer;
		url.set_path(config::PAIR_UP_URL);
		let response = client.post(url)
			.body(standard_serialize(&msg).unwrap())
			.header(reqwest::header::CONTENT_TYPE, "application/octet-stream") // Set the content type
			.send().await?;
		Ok(response.status() == StatusCode::OK)
	}
	pub async fn get_peers(client: &Client, peer: Url) -> Result<HashSet<String>, Box<dyn Error>> {
		let mut url = peer;
		url.set_path(config::GET_PEERS_URL);
		let response = client.get(url)
			.header(reqwest::header::CONTENT_TYPE, "application/octet-stream") // Set the content type
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
				Err(Box::new(e))
			}
		}
	}
}