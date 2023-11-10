use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum BlockchainError {
	InvalidTransaction,
	InvalidBlock,
	InvalidChain,
}

impl Display for BlockchainError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			BlockchainError::InvalidTransaction => {
				write!(f, "Invalid Transaction")
			}
			BlockchainError::InvalidBlock => {
				write!(f, "Invalid Block")
			}
			BlockchainError::InvalidChain => {
				write!(f, "Invalid Chain")
			}
		}
	}
}
