use actix_web::{HttpResponse, Responder, web};
use log::info;
use reqwest::Client;
use crate::core::block::{Block};
use crate::core::Hashable;
use crate::network::models::{NewBlock, NewTransaction};
use crate::network::models::http_errors::ErrorType;
use crate::network::node::Node;
use crate::network::standard::StandardExtractor;

pub async fn handle_tx(node: web::Data<Node>, msg: StandardExtractor<NewTransaction>) -> impl Responder {
	info!("Tx request received");
	
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version {
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}
	
	let transaction = &msg.transaction;
	let mut set = node.recently_seen_ids.write().await;
	
	if !set.insert(transaction.id) {
		return HttpResponse::AlreadyReported().finish();
	}
	
	let mut blockchain = node.blockchain.write().await;
	if blockchain.add_transaction_to_mempool(transaction) {
		info!("Got a new transaction. ID: \"{:?}\"", transaction.id);
		let peers = node.peers.read().await;
		Node::broadcast_transaction(&node, peers.clone(), &msg.into_inner()).await;
		HttpResponse::Ok().finish()
	} else {
		HttpResponse::BadRequest().body(ErrorType::InvalidTransaction(blockchain.get_context()).to_string())
	}
}

pub async fn handle_block(node: web::Data<Node>, msg: StandardExtractor<NewBlock>) -> impl Responder {
	// TODO: DEFENETLY CHECK THIS FUNCTION LOL
	// TODO CHECK IF THE BLOCK IS THE SAME HEIGHT AS THE CURRENT ONE AND STILL VALID
	let request_version = msg.version;
	let required_version = node.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, node.version).to_string());
	}

	let block = &msg.block;
	let mut set = node.recently_seen_ids.write().await;

	if !set.insert(block.calculate_hash()) {
		return HttpResponse::AlreadyReported().finish();
	}
	let mut blockchain = node.blockchain.write().await;
	if !block.is_correct() {
		info!("Received incorrect block");
		return HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string());
	}
	if block.header.previous_hash == blockchain.get_last_block().header.hash {
		// Is next block
		if blockchain.add_block(block) {
			info!("Received valid block with hash {} and height {}", hex::encode(block.header.hash), block.header.height);
			// TODO: Uncomment when no more testing
			node.broadcast_block(&msg.into_inner(), &node.peers.read().await.clone()).await;
			HttpResponse::Ok().finish()

		} else {
			info!("Received invalid block");
			HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string())
		}
	} else if block.header.height > blockchain.get_last_block().header.height {
		const MAX_BLOCK_LOCATOR_OBJECTS: usize = 1024;
		let peers = node.peers.read().await;
		let client = Client::new();
		let blockchain_height = blockchain.get_height();
		let mut block_locator_object = Vec::with_capacity(MAX_BLOCK_LOCATOR_OBJECTS);
		for i in blockchain_height-MAX_BLOCK_LOCATOR_OBJECTS..blockchain_height {
			if let Some(hash) = blockchain.get_block_hash_at(i) { // TODO: Check for off by 1 error
				block_locator_object.push(hash);
			}
		}
		block_locator_object.reverse(); // TODO: Check if this needs to be reversed?
		let mut was_blockchain_replaced = false;
		'peers_loop: for peer in peers.iter() {
			if let Ok(headers) = node.get_headers(peer, &client, block_locator_object.clone()).await {
				let headers = headers.headers;
				if let Some(common_header) = headers.first() {
					if blockchain.get_block_hash_at(common_header.height) != Some(common_header.hash)
						|| common_header.height == blockchain_height {
						continue
					}
					if headers.last().unwrap().height <= blockchain.get_height(){
						continue
					}
					let mut last_hash = common_header.previous_hash;
					// Check PoW of all headers
					for header in &headers {
						if !header.verify_proof_of_work(node.parameters.network_parameters.proof_of_work_difficulty) || last_hash != header.previous_hash { // TODO: Change this if dynamic PoW difficulty
							continue 'peers_loop
						}
						last_hash = header.hash;
					}

					let undone_blocks = blockchain.undo_until(common_header.previous_hash);
					if undone_blocks.is_none() {
						continue 'peers_loop
					}
					let undone_blocks = undone_blocks.unwrap();
					if let Ok(blocks) = node.get_blocks_data(
						peer,
						&client, 
						headers.clone()
							.iter()
							.map(
								|header| header.hash
							)
							.collect()
					).await {

						let data = blocks.blocks_data;
						
						if blocks.version != node.version || data.iter().any(|block_data| block_data.is_none()) { // Is invalid if version is not the same or if any block data is None
							continue 'peers_loop
						}
						let block_content_of_this_peer = data.iter().map(|block_content| block_content.clone().unwrap());
						let mut is_valid = true;
						let mut i = 0;
						for (block_content, block_header) in block_content_of_this_peer.zip(&headers) {
							let block = Block::new_raw(*block_header, block_content.clone());
							if !blockchain.add_block(&block) {
								is_valid = false;
								i+=1;
								break
							}
							i+=1;
						}
						if !is_valid {
							// Found an invalid block, undo all blocks from this peer and redo all blocks to get to snapshot
							for (j, header) in headers.into_iter().enumerate() {
								if j == i {
									break
								}
								blockchain.undo_block(&header.hash).ok();
							}
							
							for u in undone_blocks {
								blockchain.add_block(&u);
							}
							continue 'peers_loop
						}
						// The blockchain was valid and better so we replaced it
						was_blockchain_replaced = true;
					}
				
				}
			}
		}
		return if was_blockchain_replaced {
			HttpResponse::Ok().finish()
		} else {
			HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string())
		}
	} else {
		return HttpResponse::BadRequest().body(ErrorType::InvalidBlock(blockchain.get_context()).to_string());
	}
}
