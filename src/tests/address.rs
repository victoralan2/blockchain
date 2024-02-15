use crate::core::address::P2PKHAddress;

#[test]
fn address_test() {
	let addr = P2PKHAddress::random().0;
	println!("{}", addr);
	let new_addr = P2PKHAddress::from_string(addr.to_string()).expect("Unable serialize key");
	assert_eq!(new_addr, addr);
	assert_eq!(new_addr.to_string(), addr.to_string());
}
