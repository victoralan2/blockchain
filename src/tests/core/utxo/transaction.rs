use crate::core::utxo::Input;
use crate::core::utxo::transaction::Transaction;

#[test]
pub fn test_tx_signatures() {
    
    
    let input_a = Input {
        prev_txid: [0u8; 32],
        output_index: 0,
        signature: vec![],
        public_key: vec![],
    };
}