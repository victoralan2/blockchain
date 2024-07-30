use ctor::ctor;

use crate::logger::setup_logger;

#[ctor]
fn init() {
	if let Err(info) = setup_logger() {
		eprintln!("Unable to setup logger. No output will be provided. Error: {}", info);
	}
}
