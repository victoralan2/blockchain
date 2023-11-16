use actix_web::{HttpResponse, Responder, web};
use crate::network::standard::DefaultExtractor;
use crate::network::models::http_errors::ErrorType;
use crate::network::models::{NewBlock, NewTransaction};
use crate::network::node::State;

pub async fn handle_tx(state: web::Data<State>, msg: DefaultExtractor<NewTransaction>) -> impl Responder {
	let request_version = msg.version;
	let required_version = state.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, state.version).to_message());
	}
	
	let transaction = &msg.transaction;
	// TODO
	HttpResponse::Ok().finish()
}
pub async fn handle_block(state: web::Data<State>, msg: DefaultExtractor<NewBlock>) -> impl Responder {
	let request_version = msg.version;
	let required_version = state.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, state.version).to_message());
	}

	let block = &msg.block;
	// TODO
	// println!("Got block: {:?}", block);
	// state.blockchain.lock().await.push(format!("{:?}", block));
	HttpResponse::Ok().finish()
}