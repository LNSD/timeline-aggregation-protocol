// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod receipt_checks_adapter_unit_test {
    use std::{
        collections::{HashMap, HashSet},
        str::FromStr,
        sync::{Arc, RwLock},
    };

    use ethereum_types::Address;
    use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder};
    use futures::{stream, StreamExt};
    use rstest::*;

    use crate::{
        adapters::{
            receipt_checks_adapter::ReceiptChecksAdapter,
            receipt_checks_adapter_mock::ReceiptChecksAdapterMock,
        },
        eip_712_signed_message::EIP712SignedMessage,
        tap_receipt::{get_full_list_of_checks, Receipt, ReceivedReceipt},
    };

    #[rstest]
    async fn receipt_checks_adapter_test() {
        let gateway_ids = [
            Address::from_str("0xfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfb").unwrap(),
            Address::from_str("0xfafafafafafafafafafafafafafafafafafafafa").unwrap(),
            Address::from_str("0xadadadadadadadadadadadadadadadadadadadad").unwrap(),
        ];
        let gateway_ids_set = Arc::new(RwLock::new(HashSet::from(gateway_ids)));

        let allocation_ids = [
            Address::from_str("0xabababababababababababababababababababab").unwrap(),
            Address::from_str("0xbabababababababababababababababababababa").unwrap(),
            Address::from_str("0xdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdf").unwrap(),
        ];
        let allocation_ids_set = Arc::new(RwLock::new(HashSet::from(allocation_ids)));

        let wallet: LocalWallet = MnemonicBuilder::<English>::default()
         .phrase("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
         .build()
         .unwrap();
        let value = 100u128;

        let receipts: HashMap<u64, ReceivedReceipt> = stream::iter(0..10)
            .then(|id| {
                let wallet = wallet.clone();
                async move {
                    (
                        id,
                        ReceivedReceipt::new(
                            EIP712SignedMessage::new(
                                Receipt::new(allocation_ids[0], value).unwrap(),
                                &wallet,
                            )
                            .await
                            .unwrap(),
                            id,
                            &get_full_list_of_checks(),
                        ),
                    )
                }
            })
            .collect::<HashMap<_, _>>()
            .await;
        let receipt_storage = Arc::new(RwLock::new(receipts));

        let query_appraisals = (0..11)
            .into_iter()
            .map(|id| (id, 100u128))
            .collect::<HashMap<_, _>>();

        let query_appraisals_storage = Arc::new(RwLock::new(query_appraisals));

        let receipt_checks_adapter = ReceiptChecksAdapterMock::new(
            Arc::clone(&receipt_storage),
            query_appraisals_storage,
            allocation_ids_set,
            gateway_ids_set,
        );

        let new_receipt = (
            10u64,
            ReceivedReceipt::new(
                EIP712SignedMessage::new(Receipt::new(allocation_ids[0], value).unwrap(), &wallet)
                    .await
                    .unwrap(),
                10u64,
                &get_full_list_of_checks(),
            ),
        );

        let unique_receipt_id = 0u64;
        receipt_storage
            .write()
            .unwrap()
            .insert(unique_receipt_id, new_receipt.1.clone());

        assert!(receipt_checks_adapter.is_unique(&new_receipt.1.signed_receipt, unique_receipt_id));
        assert!(receipt_checks_adapter
            .is_valid_allocation_id(new_receipt.1.signed_receipt.message.allocation_id));
        // TODO: Add check when gateway_id is available from received receipt (issue: #56)
        // assert!(receipt_checks_adapter.is_valid_gateway_id(gateway_id));
        assert!(receipt_checks_adapter.is_valid_value(
            new_receipt.1.signed_receipt.message.value,
            new_receipt.1.query_id
        ));
    }
}