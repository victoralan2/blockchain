
pub enum RequestType {
	P2PDiscover,
	NewTransaction,
	NewBlock,
	SyncRequest,
	InitialBlockDownload,
}

impl TryFrom<u8> for RequestType {
	type Error = ();

	fn try_from(value: u8) -> Result<RequestType, ()> {
		match value {
			0 => {
				Ok(Self::P2PDiscover)
			},
			1 => {
				Ok(Self::NewTransaction)
			},
			2 => {
				Ok(Self::NewBlock)
			},
			3 => {
				Ok(Self::SyncRequest)
			}
			4 => {
				Ok(Self::InitialBlockDownload)
			}
			_ => {
				Err(())
			}
		}
	}
}
impl Into<u8> for RequestType {
	fn into(self) -> u8{
		match self {
			RequestType::P2PDiscover => {0}
			RequestType::NewTransaction => {1}
			RequestType::NewBlock => {2}
			RequestType::SyncRequest => {3}
			RequestType::InitialBlockDownload => {4}
		}
	}
}