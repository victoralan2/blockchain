use std::collections::HashSet;
use std::str::FromStr;

use actix_web::{HttpRequest, HttpResponse, Responder, web};
use regex::Regex;
use reqwest::Url;

use crate::network::models::{HttpScheme, PairUp, SendPeers};
use crate::network::models::http_errors::ErrorType;
use crate::network::node::Node;
use crate::network::standard;
use crate::network::standard::StandardExtractor;

pub const URL_REGEX: &str = r"(https?)://[0-9]{1,3}\.[a-zA-Z0-9]+\.[a-zA-Z0-9]+\.[a-zA-Z0-9]+:[0-9]{1,5}/[a-z, A-Z, 0-9, /]*$";

pub async fn handle_get_peers(node: web::Data<Node>) -> impl Responder {

	let peers: HashSet<String> = node.peers.read().await.iter().map(|url|url.to_string()).collect();
	if let Ok(msg) = standard::standard_serialize(&SendPeers {
		peers,
	}) {
		HttpResponse::Ok().body(msg)
	} else {
		HttpResponse::InternalServerError().finish()
	}
}
pub async fn handle_pair_up(node: web::Data<Node>, msg: StandardExtractor<PairUp>, req: HttpRequest) -> impl Responder {
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let request_port = msg.port;
	let scheme = match msg.method {
		HttpScheme::HTTP => {"http"}
		HttpScheme::HTTPS => {"https"}
	};
	if let Some(addr) = req.peer_addr() {
		let url_string = format!("{}://{}:{}", scheme, addr.ip(), request_port);
		let regex = Regex::new(URL_REGEX).unwrap();
		if regex.is_match(&url_string) { // TODO Do a check for size of peer list
			if let Ok(url) = Url::from_str(&url_string) {
				return if node.peers.read().await.len() < node.config.max_peers {
					if node.peers.read().await.iter().any(|x| x.host_str() == Some(&addr.to_string())) { // If the address is already peer
						HttpResponse::UnprocessableEntity().body("The given address is already a peer")
					} else {
						node.peers.write().await.insert(url);
						HttpResponse::Ok().finish()
					}
				} else {
					HttpResponse::InsufficientStorage().body("Peer list full")
				}
			}
			HttpResponse::BadRequest().body(ErrorType::InvalidUrl.to_string())
		} else {
			HttpResponse::BadRequest().body(ErrorType::InvalidUrl.to_string())
		}
	} else {
		println!("Internal server err");
		HttpResponse::InternalServerError().finish()
	}
}

pub async fn handle_unpair(node: web::Data<Node>, msg: StandardExtractor<PairUp>, req: HttpRequest) -> impl Responder {
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let request_port = msg.port;
	let scheme = match msg.method {
		HttpScheme::HTTP => {"http"}
		HttpScheme::HTTPS => {"https"}
	};

	if let Some(addr) = req.peer_addr() {
		let url_string = format!("{}://{}:{}", scheme, addr.ip(), request_port);
		let regex = Regex::new(URL_REGEX).unwrap();
		if regex.is_match(&url_string) {
			if let Ok(url) = Url::from_str(&url_string) {
				let mut peers = node.peers.write().await;
				if peers.remove(&url) {
					return HttpResponse::Ok().finish()
				} else {
					for p in peers.clone() {
						if let Some(host) = p.host_str() {
							if host == addr.to_string() {
								peers.remove(&p);
								return HttpResponse::Ok().finish()
							}
						}
					}
					HttpResponse::BadRequest().body("The given address was not a peer");
				}
			}
			HttpResponse::BadRequest().body(ErrorType::InvalidUrl.to_string())
		} else {
			HttpResponse::BadRequest().body(ErrorType::InvalidUrl.to_string())
		}
	} else {
		println!("Internal server err");
		HttpResponse::InternalServerError().finish()
	}
}