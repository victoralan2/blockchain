use crate::crypto::hash::hash;

pub fn calculate_merkle_root(data: Vec<[u8; 32]>) -> [u8; 32] {
	if data.is_empty() {
		return [0u8; 32];
	}

	if data.len() == 1 {
		return data[0].clone();
	}

	let mut new_data: Vec<[u8; 32]> = Vec::new();

	for chunk in data.chunks(2) {
		let mut combined_data: Vec<u8> = Vec::new();

		for chunk_data in chunk {
			combined_data.extend_from_slice(chunk_data);
		}

		let hash = hash(&combined_data);

		new_data.push(hash);
	}

	calculate_merkle_root(new_data)
}