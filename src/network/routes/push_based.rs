use actix_web::{HttpResponse, Responder, web};
use log::info;

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
		info!("Got a new transaction. TXID: \"{:?}\"", transaction.id);
		let peers = node.peers.read().await;
		Node::broadcast_transaction(peers.clone(), &msg.into_inner()).await; // TODO: Actually check for duplicates
		HttpResponse::Ok().finish()
	} else {
		HttpResponse::BadRequest().body(ErrorType::InvalidTransaction(blockchain.get_context()).to_string())
	}
}

pub async fn handle_block(node: web::Data<Node>, msg: StandardExtractor<NewBlock>) -> impl Responder {
	// TODO CHECK IF THE BLOCK IS THE SAME HEIGHT AS THE CURRENT ONE AND STILL VALID
	// TODO: CHECK IF BLOCK IS BEFORE CURRENT SLOT BUT AFTER LAST'S BLOCK SLOT
	// TODO: IF SLOT IS SAMES AS LAST BLOCK AND HEIGHT IS SAMES AS LAST BLOCK, CHECK FOR LOTTERY
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let block = &msg.block;
	let mut blockchain = node.blockchain.write().await;
	if blockchain.add_block(block) {
		info!("Received valid block");

		// TODO: Uncomment when no more testing
		// node.broadcast_block(&msg.into_inner()).await; // TODO: Actually check for duplicates
		HttpResponse::Ok().finish()
	} else {
		info!("Received invalid block");
		HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string())
	}
}
