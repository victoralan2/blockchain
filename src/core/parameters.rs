use crate::consensus::gen_difficulty;

pub const COIN_NAME: &str = "RESONANCE";
pub const COIN_NAME_ABBREVIATION: &str = "RSN";

#[derive(Clone, Copy, Default)]
pub struct Parameters {
	pub(crate) network_parameters: NetworkParameters,
	// pub(crate) technical_parameters: TechnicalParameters,
	pub(crate) economic_parameters: EconomicParameters,
}

#[derive(Copy, Clone)]
pub struct NetworkParameters {
	// Max block body size in bytes
	pub(crate) max_block_body_size: usize,
	// Max transaction size in bytes
	pub(crate) max_tx_size: usize,
	pub(crate) proof_of_work_difficulty: [u8; 32],
}

impl Default for NetworkParameters {
	fn default() -> Self {
		// TODO: Check for this parameters when receiving
		NetworkParameters {
			max_block_body_size: 2usize.pow(16), // 65536B -> 64Kib
			max_tx_size: 2usize.pow(10), // 512B
			proof_of_work_difficulty: gen_difficulty(10000000), // TODO: EXAMPLE
		}
	}
}

#[derive(Copy, Clone)]
pub struct EconomicParameters {
	pub(crate) mining_reward: u64,
}

impl Default for EconomicParameters {
	fn default() -> Self {
		Self {
			mining_reward: 100,
		}
	}
}