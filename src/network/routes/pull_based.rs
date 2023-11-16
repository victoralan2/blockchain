use actix_web::{Responder, web};
use crate::network::standard::DefaultExtractor;
use crate::network::models::{GetBlocks, GetData, GetHeaders};
use crate::network::node::State;

pub async fn handle_get_blocks(state: web::Data<State>, msg: DefaultExtractor<GetBlocks>) -> impl Responder {
	// TODO
	""
}
pub async fn handle_get_data(state: web::Data<State>, msg: DefaultExtractor<GetData>) -> impl Responder {
	// TODO
	""
}
pub async fn handle_get_headers(state: web::Data<State>, msg: DefaultExtractor<GetHeaders>) -> impl Responder {
	// TODO
	""
}
