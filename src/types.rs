use rustc_hex::{FromHex, FromHexError};
use web3::types::{Address, Block, H256};

pub fn hex_to_addr(s: &str) -> Result<Address, FromHexError> {
    s.to_owned()
        .chars()
        .skip(2)
        .collect::<String>()
        .from_hex::<Vec<u8>>()
        .map(|v| Address::from_slice(&v))
}

pub fn hex_to_h256(s: &str) -> Result<H256, FromHexError> {
    s.to_owned()
        .chars()
        .skip(2)
        .collect::<String>()
        .from_hex::<Vec<u8>>()
        .map(|v| H256::from_slice(&v))
}

pub fn into_block<T1, T2, F>(block: Block<T1>, f: F) -> Block<T2>
where
    F: FnMut(T1) -> T2,
{
    Block {
        hash: block.hash,
        parent_hash: block.parent_hash,
        uncles_hash: block.uncles_hash,
        author: block.author,
        state_root: block.state_root,
        transactions_root: block.transactions_root,
        receipts_root: block.receipts_root,
        number: block.number,
        gas_used: block.gas_used,
        gas_limit: block.gas_limit,
        extra_data: block.extra_data,
        logs_bloom: block.logs_bloom,
        timestamp: block.timestamp,
        difficulty: block.difficulty,
        total_difficulty: block.total_difficulty,
        seal_fields: block.seal_fields,
        uncles: block.uncles,
        transactions: block.transactions.into_iter().map(f).collect(),
        size: block.size,
    }
}