use pqcrypto_dilithium::dilithium5::PublicKey;

#[derive(Clone)]
pub struct Address {
	public_key: PublicKey,
	address: String
}

impl Address{
	pub fn new() -> Self {
		// TODO
		unimplemented!()
	}
	pub fn null() -> Self {
		// TODO
		unimplemented!()
	}
}