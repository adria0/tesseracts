#[cfg(test)]
mod tests {
    use super::super::appdb::*;
    use super::super::super::eth::types::*;

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    use web3::types::{Bytes, Transaction, Address, H256, TransactionReceipt,U128, U256, H2048};

    fn init() -> AppDB {
        let mut rng = thread_rng();
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();

        let mut tmpfile = std::env::temp_dir();
        tmpfile.push(chars);

        AppDB::open_default(
            tmpfile.as_os_str().to_str().expect("bad OS filename"),
            Options {
                store_itx  : true,
                store_tx   : true,
                store_addr : true,
                store_neb  : true,            
            }
        ).expect("unable to create db")
    }

    struct TestVars {
        one_u256 : U256,
        a1 : Address,
        a2 : Address,
        a3 : Address,
        a4 : Address,
        h1 : H256,
        h2 : H256,
        h3 : H256,
        tx_a1_to_a2 : Transaction,
        rcp_a1_to_a2 : TransactionReceipt,
        tx_a1_to_contract : Transaction,
        rcp_a1_to_contract : TransactionReceipt,
        tx_a1_to_a1 : Transaction,
        rcp_a1_to_a1 : TransactionReceipt,
    }

    fn vars() -> TestVars {
        let one_u128 = U128::from_dec_str("1").unwrap();
        let one_u256 = U256::from_dec_str("1").unwrap();
        let a1 = hex_to_addr("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        let a2 = hex_to_addr("0bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
        let a3 = hex_to_addr("0xcccccccccccccccccccccccccccccccccccccccc").unwrap();
        let a4 = hex_to_addr("0xdddddddddddddddddddddddddddddddddddddddd").unwrap();
        let h1 = hex_to_h256("0x1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
        let h2 = hex_to_h256("0x2222222222222222222222222222222222222222222222222222222222222222")
            .unwrap();
        let h3 = hex_to_h256("0x3333333333333333333333333333333333333333333333333333333333333333")
            .unwrap();

        let tx_a1_to_a2 = Transaction {
            hash: h1,
            nonce: one_u256,
            block_hash: None,
            block_number: Some(U256::from_dec_str("10").unwrap()),
            transaction_index: Some(one_u128),
            from: a1,
            to: Some(a2),
            value: one_u256,
            gas_price: one_u256,
            gas: one_u256,
            input: Bytes(Vec::new()),
        };
        let rcp_a1_to_a2 = TransactionReceipt {
            block_hash: None,
            block_number: tx_a1_to_a2.block_number,
            transaction_index: one_u128,
            contract_address: None,
            gas_used: Some(one_u256),
            cumulative_gas_used: one_u256,
            status: None,
            transaction_hash: tx_a1_to_a2.hash,
            logs: Vec::new(),
            logs_bloom: H2048::default()
        };

        let tx_a1_to_contract = Transaction {
            hash: h2,
            nonce: one_u256,
            block_hash: None,
            block_number: Some(U256::from_dec_str("11").unwrap()),
            transaction_index: Some(one_u128),
            from: a1,
            to: None,
            value: one_u256,
            gas_price: one_u256,
            gas: one_u256,
            input: Bytes(Vec::new()),
        };
        let rcp_a1_to_contract = TransactionReceipt {
            block_hash: None,
            block_number: tx_a1_to_contract.block_number,
            transaction_index: one_u128,
            contract_address: Some(a3),
            gas_used: Some(one_u256),
            cumulative_gas_used: one_u256,
            status: None,
            transaction_hash: tx_a1_to_contract.hash,
            logs: Vec::new(),
            logs_bloom: H2048::default()
        };
        let tx_a1_to_a1 = Transaction {
            hash: h3,
            nonce: one_u256,
            block_hash: None,
            block_number: Some(U256::from_dec_str("12").unwrap()),
            transaction_index: Some(one_u128),
            from: a1,
            to: Some(a1),
            value: one_u256,
            gas_price: one_u256,
            gas: one_u256,
            input: Bytes(Vec::new()),
        };
        let rcp_a1_to_a1 = TransactionReceipt {
            block_hash: None,
            block_number: tx_a1_to_a1.block_number,
            transaction_index: one_u128,
            contract_address: None,
            gas_used: Some(one_u256),
            cumulative_gas_used: one_u256,
            status: None,
            transaction_hash: tx_a1_to_a1.hash,
            logs: Vec::new(),
            logs_bloom: H2048::default()
        };
        TestVars {
            one_u256,
            a1,a2,a3,a4,h1,h2,h3,
            tx_a1_to_a2,rcp_a1_to_a2,
            tx_a1_to_contract,rcp_a1_to_contract,
            tx_a1_to_a1, rcp_a1_to_a1 
        }
    }

    #[test]    
    fn test_add_and_iter_tx() {
        let appdb = init();
        let v = vars();

        // no txs

        assert_eq!(0, appdb.count_addr_tx_links(&v.a1).unwrap());
        assert_eq!(0, appdb.count_addr_tx_links(&v.a2).unwrap());
        assert_eq!(0, appdb.count_addr_tx_links(&v.a3).unwrap());

        // + tx_a1_to_a2

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a2, &v.rcp_a1_to_a2, Some(&[])).unwrap());
        assert_eq!(1, appdb.count_addr_tx_links(&v.a1).unwrap());
        assert_eq!(1, appdb.count_addr_tx_links(&v.a2).unwrap());

        let mut it_a1 = appdb.iter_addr_tx_links(&v.a1);
        assert_eq!(Some((v.h1,0)), it_a1.next());
        assert_eq!(None, it_a1.next());

        let mut it_a2 = appdb.iter_addr_tx_links(&v.a2);
        assert_eq!(Some((v.h1,0)), it_a2.next());
        assert_eq!(None, it_a2.next());

        // + tx_a1_to_contract

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_contract, &v.rcp_a1_to_contract,Some(&[])).unwrap());
        assert_eq!(2, appdb.count_addr_tx_links(&v.a1).unwrap());
        assert_eq!(1, appdb.count_addr_tx_links(&v.a2).unwrap());
        assert_eq!(1, appdb.count_addr_tx_links(&v.a3).unwrap());

        let mut it_a1 = appdb.iter_addr_tx_links(&v.a1);
        assert_eq!(Some((v.h2,0)), it_a1.next());
        assert_eq!(Some((v.h1,0)), it_a1.next());
        assert_eq!(None, it_a1.next());

        let mut it_a3 = appdb.iter_addr_tx_links(&v.a3);
        assert_eq!(Some((v.h2,0)), it_a3.next());
        assert_eq!(None, it_a3.next());

        // + tx_a1_to_a1

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a1, &v.rcp_a1_to_a1,Some(&[])).unwrap());
        assert_eq!(3, appdb.count_addr_tx_links(&v.a1).unwrap());

        let mut it_a1 = appdb.iter_addr_tx_links(&v.a1);
        assert_eq!(Some((v.h3,0)), it_a1.next());
        assert_eq!(Some((v.h2,0)), it_a1.next());
        assert_eq!(Some((v.h1,0)), it_a1.next());
        assert_eq!(None, it_a1.next());
    }

    #[test]
    fn test_add_and_iter_itx_a1_to_a2() {

        let appdb = init();
        let v = vars();

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a2, &v.rcp_a1_to_a2, Some(&[
            InternalTx { from : v.a2, to: Some(v.a3), contract:None, input: Vec::new(), value:v.one_u256 }
        ])).unwrap());

        let mut it_a2 = appdb.iter_addr_tx_links(&v.a2);
        assert_eq!(Some((v.h1,1)), it_a2.next());
        assert_eq!(Some((v.h1,0)), it_a2.next());
        assert_eq!(None, it_a2.next());
    }

    #[test]
    fn test_add_and_iter_itx_a2_to_contract() {

        let appdb = init();
        let v = vars();

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a2, &v.rcp_a1_to_a2, Some(&[
            InternalTx { from : v.a2, to: None, contract:Some(v.a4), input: Vec::new(), value:v.one_u256}
        ])).unwrap());

        let mut it_a2 = appdb.iter_addr_tx_links(&v.a2);
        assert_eq!(Some((v.h1,1)), it_a2.next());
        assert_eq!(Some((v.h1,0)), it_a2.next());
        assert_eq!(None, it_a2.next());

        let mut it_a4 = appdb.iter_addr_tx_links(&v.a4);
        assert_eq!(Some((v.h1,1)), it_a4.next());
        assert_eq!(None, it_a4.next());
    }

    #[test]
    fn test_add_and_iter_itx_a1_to_a1() {
        let appdb = init();
        let v = vars();

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a1, &v.rcp_a1_to_a1, Some(&[
            InternalTx { from : v.a1, to: Some(v.a1), contract:None, input: Vec::new(), value:v.one_u256 }
        ])).unwrap());

        let mut it_a1 = appdb.iter_addr_tx_links(&v.a1);
        assert_eq!(Some((v.h3,1)), it_a1.next());
        assert_eq!(Some((v.h3,0)), it_a1.next());
        assert_eq!(None, it_a1.next());
    }

    #[test]
    fn test_add_and_iter_itx_a1_to_a1_2itx() {
        let appdb = init();
        let v = vars();

        assert_eq!((), appdb.add_tx(&v.tx_a1_to_a1, &v.rcp_a1_to_a1, Some(&[
            InternalTx { from : v.a3, to: Some(v.a1), contract:None, input: Vec::new(), value:v.one_u256 },
            InternalTx { from : v.a2, to: None, contract:Some(v.a3), input: Vec::new(), value:v.one_u256 }
        ])).unwrap());

        assert_eq!(0,appdb._count_itxs(&v.h1));
        assert_eq!(2,appdb._count_itxs(&v.h3));

        let mut i_itx = appdb.iter_itxs(&v.h3);
        assert_eq!(i_itx.next().map(|(n,itx)| (n,itx.from,itx.contract)),Some((2,v.a2,Some(v.a3))));
        assert_eq!(i_itx.next().map(|(n,itx)| (n,itx.from,itx.to)),Some((1,v.a3,Some(v.a1))));
        assert_eq!(None, i_itx.next());
    }

    #[test]
    fn test_set_get_block() {
        let appdb = init();
        assert_eq!(Ok(None), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(1));
        assert_eq!(Ok(Some(1)), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(0xaabbccdd11223344));
        assert_eq!(Ok(Some(0xaabbccdd11223344)), appdb.get_last_block());
    }

}
