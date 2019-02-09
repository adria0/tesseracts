use rlp::RlpStream;
use keccak_hash::keccak;
use ethkey::{Signature,recover};
use web3::types::{H256, Address};

    /* example header 
        -----------------------------------------------
        Block:      7269
        vanity:     d683010810846765746886676f312e3131856c696e7578000000000000000000
        sig:        c5b1f7978f1ac8acfdf7fd7c3b1cd0c154d4bd934e0c8ec6c17c6de160c42d1c
                    218cce60293226fee8bdb5292720afd452932706ad673f707eb3486a8a8084e8
                    00
        Hash:       0x6adaaeb02efa0aced16e5931a2935b43da653d12e978c130bbdda1fb6214ff7e
        rtl:        f90216a012e8a425926ba32afd3d55e0dbdf083de02e96845041914129e3d03e3bd461faa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347940000000000000000000000000000000000000000a0093b304253d9f02698a68d410803c4cf922b8a12f9f15ceed0208ec31cd1f431a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001821c65837a120080845c087102a0d683010810846765746886676f312e3131856c696e7578000000000000000000a00000000000000000000000000000000000000000000000000000000000000000880000000000000000
        Signature:  332dfeffacf8eb84d09fc22df6862673b57ad721648c81c577976513386bd80a4c3c581664196dd83866165ae6f57fff59b31d63788867cf2ac77904d09c5b4901
        pubkey:     043665cb2b553b888403b7b16107159804af66095ef4b50e7e2cbaa9ef1a5294445f86cfc4a82ff816786cfc0c780bb492cc7c36ca4662a04790a9fbe56e330a1e
        signer:     0xF979Deb61F2E982761Ac22b2a647cafDc5263fFc
        -----------------------------------------------
    */


pub fn parse_clique_signer<T>(block : &web3::types::Block<T>) -> Option<Address> {
    const EXTRA_SEAL : usize = 65;
    let vanity = &block.extra_data.0[..block.extra_data.0.len()-EXTRA_SEAL];
    let mut stream = RlpStream::new_list(15);
    stream
		.append(&block.parent_hash)
		.append(&block.uncles_hash)
		.append(&block.author)
		.append(&block.state_root)
		.append(&block.transactions_root)
		.append(&block.receipts_root)
		.append(&block.logs_bloom)
		.append(&block.difficulty)
		.append(&block.number.unwrap())
		.append(&block.gas_limit)
		.append(&block.gas_used)
		.append(&block.timestamp)
		.append(&vanity.to_vec())
		.append(&block.mix_hash.unwrap())
		.append(&block.nonce.unwrap());
        
    let rlp = &stream.out();

    let seal_hash = keccak(&rlp);

    let signature = &block.extra_data.0[block.extra_data.0.len()-EXTRA_SEAL..];

    let r = H256::from_slice(&signature[0..32]);
    let s = H256::from_slice(&signature[32..]);
    let v = signature[64];

    let sig = Signature::from_rsv(&r, &s, v);
 
    if sig.is_valid() {
        if let Ok(pbk) = recover(&sig, &seal_hash) {
            let pbk_hash = keccak(pbk);
            let signer = Address::from_slice(&pbk_hash.0[12..]);
            return Some(signer);
        }
    }
    None
}
