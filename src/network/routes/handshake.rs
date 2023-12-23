use actix_web::{Responder, web};
use crate::network::node::Node;


pub async fn handle_version(node: web::Data<Node>) -> impl Responder {
	node.version.to_string()
}