use actix_web::{HttpResponse, Responder, web};

use crate::network::models::{NewBlock, NewTransaction};
use crate::network::models::http_errors::ErrorType;
use crate::network::node::Node;
use crate::network::standard::StandardExtractor;

pub async fn handle_tx(node: web::Data<Node>, msg: StandardExtractor<NewTransaction>) -> impl Responder {
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}
	
	let transaction = &msg.transaction;
	let mut blockchain = node.blockchain.write().await;
	if blockchain.add_transaction_to_mempool(transaction) {
		println!("Got transaction: {:?}", transaction);
		let peers = node.peers.read().await;
		Node::broadcast_transaction(peers.clone(), &msg.into_inner()).await; // TODO: Actually check for duplicates
		HttpResponse::Ok().finish()
	} else {
		HttpResponse::BadRequest().body(ErrorType::InvalidTransaction(blockchain.get_context()).to_string())
	}
}

pub async fn handle_block(node: web::Data<Node>, msg: StandardExtractor<NewBlock>) -> impl Responder {
	// TODO CHECK IF THE BLOCK IS THE SAME HEIGHT AS THE CURRENT ONE AND STILL VALID
	// TODO: CHECK IF BLOCK IS THE SAME SLOT AS CURRENT
	// TODO: IF BLOCK IS THE SAME SLOT AND THE SAME HEIGHT CHECK IF LOTTERY NUMBER IS SMALLER IN THIS ONE

	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let block = &msg.block;
	let mut blockchain = node.blockchain.write().await;
	if blockchain.add_block(block) {
		println!("Got block: {:?}", block);
		let peers = node.peers.read().await;
		Node::broadcast_block(peers.clone(), &msg.into_inner()).await; // TODO: Actually check for duplicates
		HttpResponse::Ok().finish()
	} else {
		HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string())
	}
}