use std::arch::x86_64::_mm_undefined_si128;
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::Shutdown::Both;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::core::block::Block;
use crate::core::blockchain::BlockChain;

pub fn sync_in(mut connection: TcpStream, blockchain: Arc<Mutex<BlockChain>>) -> Result<(), Error> {
	connection.write_all(b"get_chain")?;
	let mut response = Vec::new();
	connection.read_to_end(&mut response).ok();
	if response == b"get" {
		// First we need to know where the peer needs the blocks
		connection.write_all(&blockchain.lock().expect("Unable to lock").get_len().to_be_bytes())?;
		let mut size = [0u8; 8];
		connection.read_exact(&mut size).ok();
		let other_length = usize::from_be_bytes(size);
		let mut index = other_length - 1;
		let mut hash = [0u8; 32];
		loop { // Loop until we find matching block
			connection.write_all(format!("hash_at_{}", index).as_bytes())?;
			connection.read_exact(&mut hash).expect("Unable to read");
			if index == 0 {
				break;
			}
			let blockchain = blockchain.lock().expect("Unable to lock");
			let block = blockchain.get_block_at(index);
			if let Some(block) = block {
				if block.hash == hash {
					break;
				}
			} else {
				break;
			}
			index -= 1;
		}
		// We found a matching hash
		connection.write_all(format!("found_{}", index).as_bytes()).unwrap();
		connection.write_all(&hash).unwrap();
		let mut response = Vec::new();
		connection.read_to_end(&mut response).unwrap();
		while response.starts_with(b"get_") {
			let blockchain = blockchain.lock().expect("Unable to lock");
			let idx = String::from_utf8(response[4..].to_vec()).unwrap().parse::<usize>().expect("Unable to parse");
			let block = blockchain.get_block_at(idx);
			if let Some(block) = block {
				let data = bincode::serialize(block).expect("Unable to serialize");
				connection.write_all(&data)?;
			} else {
				connection.write_all(b"not_found")?;
			}
			connection.read_to_end(&mut response)?;
		}
		if response == b"sync_ok" {
			connection.write_all(b"ok_ack")?;
		}
	}
	connection.shutdown(Both).ok();
	Ok(())
}

pub fn sync_out(mut connection: TcpStream, blockchain: &BlockChain) -> Result<BlockChain, io::Error> {
	let mut buff = Vec::new();
	connection.read_to_end(&mut buff)?;
	if buff != b"get_chain" {
		return Err(io::Error::new(ErrorKind::ConnectionReset, "Invalid response"));
	}
	connection.write_all(b"get")?;
	// First we need to know where the peer needs the blocks
	let mut others_length_data = [0u8; 8];
	connection.read_exact(&mut others_length_data)?;
	let other_length = usize::from_be_bytes(others_length_data);
	connection.write_all(&blockchain.get_len().to_be_bytes())?;
	let mut request = Vec::new();
	connection.read_to_end(&mut request)?;
	while request.starts_with(b"hash_at_") { // Loop until we find matching block
		if let Ok(index_str) = String::from_utf8(request[8..].to_vec()) {
			if let Ok(index) = index_str.parse() {
				let hash = blockchain.get_block_at(index).expect("Unable to get block").hash;
				connection.write_all(&hash)?;
				connection.read_to_end(&mut request)?;
			}
		}
	}
	let mut success = false;
	let mut idx = 0;
	if request.starts_with(b"found_") {
		idx = String::from_utf8(request[7..].to_vec()).unwrap().parse().unwrap();
		let mut hash = [0u8; 32];
		connection.read_exact(&mut hash)?;
		if let Some(block) = blockchain.get_block_at(idx) {
			if hash == block.hash {
				success = true;
			}
		}
	}
	if !success { // If failed to verify hash
		return Err(Error::new(ErrorKind::ConnectionReset, "Connection reset"));
	}
	let mut new_blockchain = blockchain.clone();
	new_blockchain.truncate(idx); // Make a clone of the blockchain

	for i in idx..other_length {
		connection.write_all(format!("get_{}", i).as_bytes())?;
		let mut next_block_data = Vec::new();
		connection.read_to_end(&mut next_block_data)?;
		if next_block_data == b"not_found" {
			if new_blockchain.get_len() <= blockchain.get_len() {
				return Err(Error::new(ErrorKind::InvalidInput, "Blockchain is the same length"));
			} else {
				return Ok(new_blockchain);
			}
		}
		if let Ok(next_block) = bincode::deserialize::<Block>(&next_block_data) {
			if next_block.is_valid(&new_blockchain) {} else {
				connection.write_all(b"disconnect")?;
				connection.shutdown(Both)?;
				return Err(Error::new(ErrorKind::InvalidInput, "Block wasn't valid in the current context"));
			}
		}
	}
	Ok(new_blockchain)
}