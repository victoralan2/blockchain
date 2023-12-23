
#[derive(Clone, Copy, Default)]
pub struct Parameters {
	pub(crate) network_parameters: NetworkParameters,
	pub(crate) technical_parameters: TechnicalParameters,
	pub(crate) economic_parameters: EconomicParameters,
}

#[derive(Clone, Copy)]
pub struct TechnicalParameters {
	pub(crate) slot_duration: u32, // Time in milliseconds
	pub(crate) epoch_duration: u32, // Time in slot times
	pub(crate) active_slot_probability: f32, // The possibility of a slot having a leader
}
impl Default for TechnicalParameters {
	fn default() -> Self {
		TechnicalParameters { // TODO: Fix this default parameters
			slot_duration: 1000, // 1 Second
			epoch_duration: 86400, // 1 Day
			active_slot_probability: 0.1, // 10%
		}
	}
}
#[derive(Copy, Clone)]
pub struct NetworkParameters {
	pub(crate) max_block_body_size: u32, // Max block body size
	pub(crate) max_block_header_size: u32, // Max block header size
	pub(crate) max_tx_size: u32, // Max transaction size
}
impl Default for NetworkParameters {
	fn default() -> Self {
		// TODO: Fix this default parameters
		NetworkParameters {
			max_block_body_size: 10u32.pow(16), // 65536B -> 64Kib
			max_block_header_size: 2u32.pow(10), // 1024B -> 1KiB
			max_tx_size: 2u32.pow(10), // 512B
		}
	}
}
#[derive(Copy, Clone)]
pub struct EconomicParameters {
	pub(crate) fee_per_tx_byte: u32,
}

impl Default for EconomicParameters {
	fn default() -> Self {
		// TODO: Fix this default parameters
		Self {
			fee_per_tx_byte: 10,
		}
	}
}