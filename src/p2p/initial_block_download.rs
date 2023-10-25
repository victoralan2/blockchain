use std::io;
use tokio::net::TcpStream;
use crate::core::block::Block;

pub fn download_full_block(connection: &mut TcpStream, height: u64) -> Result<Block, io::Error> {
}