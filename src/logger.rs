use std::{env, fs};
use std::fmt::Display;
use std::io::stdout;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

const LOGS_PATH: &str = "./logs/"; // TODO: Make this modifiable with command arguments


pub fn setup_logger() -> Result<(), fern::InitError> {
	fs::create_dir(LOGS_PATH).ok();
	let colors_config = ColoredLevelConfig::new()
		.error(Color::Red)
		.warn(Color::Yellow)
		.trace(Color::BrightBlack)
		.info(Color::BrightWhite)
		.debug(Color::Green);

	let file_timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
	fern::Dispatch::new()
		.level(LevelFilter::Debug)
		.filter(|metadata| metadata.target().starts_with("blockchain")) // TODO: CHANGE THIS IF THE NAME CHANGES
		.chain(
			fern::Dispatch::new()
				.format(|out, message, record| {
				let time = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
				out.finish(format_args!(
					"[{} {} {} {}] {}",
					time,
					record.level(),
					record.file().unwrap_or(""),
					record.target(),
					message
				))
			}).chain(fern::log_file(format!("{}/{}.log", LOGS_PATH, file_timestamp))?) // TODO: Change the creation of the file to a fixed place
		)
		.chain(fern::Dispatch::new()
			.format(move |out, message, record| {
				let colored_level = colors_config.color(record.level());
				let time = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
				out.finish(format_args!(
					"[{} {} {} {}] {}",
					time,
					colored_level,
					record.file().unwrap_or(""),
					record.target(),
					message
				))
			})
			.chain(stdout())
		)
		.apply()?;

	// Hook panics to log them
	std::panic::set_hook(Box::new(|panic_info| {
		// let panic_payload = panic_info.payload().downcast_ref::<String>();
		// let panic_location = if let Some(location) = panic_info.location() {
		// 	format!("{}:{}", location.file(), location.line().to_string())
		// } else {
		// 	"unknown".to_string()
		// };
		// 
		// let panic_message = match panic_payload {
		// 	Some(message) => message.clone(),
		// 	None => "unknown".to_string(),
		// };
		log::error!("{}", panic_info);
	}));

	Ok(())
}