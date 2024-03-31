use actix_web::{HttpResponse, Responder, web};

use crate::core::block::BlockContent;
use crate::network::models::{BlockchainInfo, BlocksData, GetBlocks, GetData, GetHeaders, Headers, Inv, InvDataType};
use crate::network::models::http_errors::ErrorType;
use crate::network::node::Node;
use crate::network::standard::{standard_serialize, StandardExtractor};

pub async fn handle_get_blockchain_info(node: web::Data<Node>) -> impl Responder {
	let chain = node.blockchain.read().await;
	let info = BlockchainInfo {
		version: node.version,
		height: chain.get_height(),
		best_block_header: chain.get_last_block().header,
		mempool_size: chain.mempool.len()
	};
	if let Ok(serialized) = standard_serialize(&info) {
		HttpResponse::Ok().body(serialized)
	} else {
		HttpResponse::InternalServerError().finish()
	}
}

pub async fn handle_get_blocks(node: web::Data<Node>, msg: StandardExtractor<GetBlocks>) -> impl Responder {
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let blockchain = node.blockchain.read().await;
	let last_known_blocks = msg.block_locator_object.clone();
	let hashes = blockchain.get_blocks(&last_known_blocks);

	if let Ok(msg) = standard_serialize(&Inv {
		data_type: InvDataType::Block,
		hashes,
	}) {
		HttpResponse::Ok().body(msg)
	} else {
		HttpResponse::InternalServerError().finish()
	}
}

pub async fn handle_get_data(node: web::Data<Node>, msg: StandardExtractor<GetData>) -> impl Responder {

	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}
	
	let blockchain = node.blockchain.read().await;
	let data_type = msg.data_type;
	let requested_data = &msg.hashes;
	
	match data_type {
		InvDataType::Transaction => {
			// TODO
			HttpResponse::Ok().finish()
		}
		InvDataType::Block => {
			let mut data = vec![];
			for &hash in requested_data {
				let block = blockchain.get_block_by(hash).map(|x| x.transactions.clone() as BlockContent);
				data.push(block);
			}
			if let Ok(msg) = standard_serialize(&BlocksData {
				version: node.version,
				blocks_data: data,
			}) {
				HttpResponse::Ok().body(msg)
			} else {
				HttpResponse::InternalServerError().finish()
			}
		}
	}
}
pub async fn handle_get_headers(node: web::Data<Node>, msg: StandardExtractor<GetHeaders>) -> impl Responder {

	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let blockchain = node.blockchain.read().await;
	let last_known_blocks = msg.block_locator_object.clone();
	let headers = blockchain.get_headers(&last_known_blocks);

	if let Ok(msg) = standard_serialize(&Headers {
		headers,
	}) {
		HttpResponse::Ok().body(msg)
	} else {
		HttpResponse::InternalServerError().finish()
	}
}
