use std::str::FromStr;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use regex::{Regex, RegexSet};
use reqwest::Url;
use crate::network::models::http_errors::ErrorType;
use crate::network::models::{HttpScheme, Subscribe};
use crate::network::node::State;
use crate::network::standard;
use crate::network::standard::DefaultExtractor;


pub const URL_REGEX: &str = r"(https?)://[0-9]{1,3}\.[a-zA-Z0-9]+\.[a-zA-Z0-9]+\.[a-zA-Z0-9]+:[0-9]{1,5}/[a-z, A-Z, 0-9, /]*$";

pub async fn handle_get_peers(state: web::Data<State>) -> impl Responder {
	let list: Vec<String> = state.subscribers.lock().await.iter().map(|url|url.to_string()).collect();
	if let Ok(list) = standard::serialize(&list) {
		HttpResponse::Ok().body(list) // TODO: Respond with appropriated model
	} else {
		HttpResponse::InternalServerError().finish()
	}
}
pub async fn handle_subscribe(state: web::Data<State>, msg: DefaultExtractor<Subscribe>, req: HttpRequest) -> impl Responder {

	let request_version = msg.version;
	let required_version = state.version;
	if request_version != required_version { // TODO: Make version compatibility
		return HttpResponse::BadRequest().body(ErrorType::WrongVersion(request_version, state.version).to_message());
	}

	let request_port = msg.port;
	let scheme = match msg.method {
		HttpScheme::HTTP => {"http"}
		HttpScheme::HTTPS => {"https"}
	};
	let subdirectory = &msg.subdirectory;
	if let Some(addr) = req.peer_addr() {
		let url = Url::from_str(&format!("{}://{}:{}/{}", scheme, addr.ip().to_string(), request_port, subdirectory)).unwrap(); // TODO: Test this
		let regex = Regex::new(URL_REGEX).unwrap();
		if regex.is_match(url.as_ref()) {
			state.subscribers.lock().await.push(url);
			HttpResponse::Ok().finish()
		} else {
			HttpResponse::BadRequest().finish()
		}
	} else {
		HttpResponse::InternalServerError().finish()
	}
}