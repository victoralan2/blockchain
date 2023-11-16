use std::sync::Arc;
use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{Query, ServiceConfig};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::network::node::State;
use crate::network::routes::{handshake, p2p, pull_based, push_based};




pub fn config_routes(config: &mut ServiceConfig) {
	config
		.route("/test", web::get().to(test))
		.route("/version", web::get().to(handshake::handle_version))
		.route("/tx", web::post().to(push_based::handle_tx))
		.route("/block", web::post().to(push_based::handle_block))
		.route("/subscribe", web::post().to(p2p::handle_subscribe))
		.route("/get-peers", web::get().to(p2p::handle_get_peers))
		.route("/get-blocks", web::get().to(pull_based::handle_get_blocks))
		.route("/get-data", web::get().to(pull_based::handle_get_data))
		.route("/get-headers", web::get().to(pull_based::handle_get_headers));
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TestQuery {
	version: u32,
	string: String,
}

async fn test(state: web::Data<Arc<Mutex<State>>>, query: Query<TestQuery>) -> impl Responder {
	println!("NEW REQUEST");
	HttpResponse::Ok().body("Ok")
}