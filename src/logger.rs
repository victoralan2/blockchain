use std::env;
use std::fmt::Display;
use std::io::stdout;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;


const LOGS_PATH: &str = "./logs/"; // TODO: Make this modifiable with command arguments


pub fn setup_logger() -> Result<(), fern::InitError> {
	let colors_config = ColoredLevelConfig::new()
		.error(Color::Red)
		.warn(Color::Yellow)
		.trace(Color::BrightBlack)
		.info(Color::BrightWhite)
		.debug(Color::Green);

	let file_timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
	fern::Dispatch::new()
		.level(LevelFilter::Debug)
		.filter(|metadata| metadata.target().starts_with("blockchain"))
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
	Ok(())
}