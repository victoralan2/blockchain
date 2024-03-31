use ctor::ctor;

use crate::logger::setup_logger;

#[ctor]
fn init() {
	if setup_logger().is_err() {
		eprintln!("Unable to setup logger. No output will be provided")
	}
}
