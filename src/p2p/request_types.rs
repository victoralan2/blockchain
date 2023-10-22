
pub enum RequestType {
	P2PDiscover,
	NewBlockData,
	NewBlock,
	SyncRequest,
}

impl TryFrom<Vec<u8>> for RequestType {
	type Error = ();

	fn try_from(value: Vec<u8>) -> Result<RequestType, ()> {
		if let Ok(str) = String::from_utf8(value) {
			match str.as_str() {
				"P2PDiscover" => {
					Ok(Self::P2PDiscover)
				},
				"NewBlockData" => {
					Ok(Self::NewBlockData)
				},
				"NewBlock" => {
					Ok(Self::NewBlock)
				},
				"SyncRequest" => {
					Ok(Self::SyncRequest)
				}
				_ => {
					return Err(())
				}
			}
		} else {
			Err(())
		}
	}
}