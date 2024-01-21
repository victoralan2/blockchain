use actix_bincode::BincodeSerde;
use actix_web::{HttpResponse, Responder, web};
use actix_web::web::ServiceConfig;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::network::routes::{handshake, p2p, pull_based, push_based};

pub const VERSION_URL: &str = "/version";
pub const GET_BLOCKCHAIN_INFO_URL: &str = "/get-blockchain-info";
pub const NEW_TRANSACTION_URL: &str = "/tx";
pub const NEW_BLOCK_URL: &str = "/block";
pub const PAIR_UP_URL: &str = "/pair_up";
pub const UNPAIR_URL: &str = "/unpair";
pub const GET_PEERS_URL: &str = "/get-peers";
pub const GET_BLOCKS_URL: &str = "/get-blocks";
pub const GET_DATA_URL: &str = "/get-data";

pub const GET_HEADERS_URL: &str = "/get-headers";
pub fn config_routes(config: &mut ServiceConfig) {
	config
		.route("/test", web::post().to(test))
		.route(VERSION_URL, web::get().to(handshake::handle_version))
		.route(GET_BLOCKCHAIN_INFO_URL, web::get().to(pull_based::handle_get_blockchain_info))
		.route(NEW_TRANSACTION_URL, web::post().to(push_based::handle_tx))
		.route(NEW_BLOCK_URL, web::post().to(push_based::handle_block))
		.route(PAIR_UP_URL, web::post().to(p2p::handle_pair_up))
		.route(UNPAIR_URL, web::delete().to(p2p::handle_unpair))
		.route(GET_PEERS_URL, web::get().to(p2p::handle_get_peers))
		.route(GET_BLOCKS_URL, web::get().to(pull_based::handle_get_blocks))
		.route(GET_DATA_URL, web::get().to(pull_based::handle_get_data))
		.route(GET_HEADERS_URL, web::get().to(pull_based::handle_get_headers));
}

// #[derive(Clone, Deserialize, Serialize)]
// pub struct TestQuery {
// 	version: u32,
// 	string: String,
// }

#[derive(Clone, Deserialize, Serialize, Encode, Decode)]
pub struct TestBody {
	pub(crate) test: String,
}
async fn test(msg: BincodeSerde<TestBody>) -> impl Responder {
	println!("NEW REQUEST: {}", msg.test);
	HttpResponse::Ok().body("Ok")
}