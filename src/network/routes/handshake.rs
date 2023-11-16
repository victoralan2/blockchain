use actix_web::{get, Responder, web};
use bincode::serialize;
use crate::network::node::State;


pub async fn handle_version(state: web::Data<State>) -> impl Responder {
	state.version.to_string()
}