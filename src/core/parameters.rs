pub const COIN_NAME: &str = "TENSOR";
pub const COIN_NAME_ABBREVIATION: &str = "TNS";

#[derive(Clone, Copy, Default)]
pub struct Parameters {
	pub(crate) network_parameters: NetworkParameters,
	pub(crate) technical_parameters: TechnicalParameters,
	pub(crate) economic_parameters: EconomicParameters,
}

#[derive(Clone, Copy)]
pub struct TechnicalParameters {
}

impl Default for TechnicalParameters {
	fn default() -> Self {
		TechnicalParameters { // TODO: Fix this default parameters
		}
	}
}

#[derive(Copy, Clone)]
pub struct NetworkParameters {
	// Max block body size in bytes
	pub(crate) max_block_body_size: usize,
	// Max transaction size in bytes
	pub(crate) max_tx_size: usize,
}

impl Default for NetworkParameters {
	fn default() -> Self {
		// TODO: Fix this default parameters
		// TODO: Check for this parameters when receiving
		NetworkParameters {
			max_block_body_size: 2usize.pow(16), // 65536B -> 64Kib
			max_tx_size: 2usize.pow(10), // 512B
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
		// TODO: Actually use this parameter
		Self {
			fee_per_tx_byte: 10,
		}
	}
}