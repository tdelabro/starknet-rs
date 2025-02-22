use starknet_accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet_core::{
    chain_id,
    types::{AddTransactionResultCode, ContractArtifact, FieldElement},
    utils::get_selector_from_name,
};
use starknet_providers::SequencerGatewayProvider;
use starknet_signers::{LocalWallet, SigningKey};
use std::sync::Arc;

#[tokio::test]
async fn can_get_nonce() {
    let provider = SequencerGatewayProvider::starknet_alpha_goerli();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "02da37a17affbd2df4ede7120dae305ec36dfe94ec96a8c3f49bbf59f4e9a9fa",
    )
    .unwrap();

    let account = SingleOwnerAccount::new(provider, signer, address, chain_id::TESTNET);

    assert_ne!(account.get_nonce().await.unwrap(), FieldElement::ZERO);
}

#[tokio::test]
async fn can_estimate_fee() {
    let provider = SequencerGatewayProvider::starknet_alpha_goerli();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "02da37a17affbd2df4ede7120dae305ec36dfe94ec96a8c3f49bbf59f4e9a9fa",
    )
    .unwrap();
    let tst_token_address = FieldElement::from_hex_be(
        "07394cbe418daa16e42b87ba67372d4ab4a5df0b05c6e554d158458ce245bc10",
    )
    .unwrap();

    let account = SingleOwnerAccount::new(provider, signer, address, chain_id::TESTNET);

    let fee_estimate = account
        .execute(vec![
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("1000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("2000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
        ])
        .estimate_fee()
        .await
        .unwrap();

    assert!(fee_estimate.overall_fee > 0);
}

#[tokio::test]
async fn can_simulate_execution() {
    // Simulates the tx in `can_execute_tst_mint()` without actually sending

    let provider = SequencerGatewayProvider::starknet_alpha_goerli();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "02da37a17affbd2df4ede7120dae305ec36dfe94ec96a8c3f49bbf59f4e9a9fa",
    )
    .unwrap();
    let tst_token_address = FieldElement::from_hex_be(
        "07394cbe418daa16e42b87ba67372d4ab4a5df0b05c6e554d158458ce245bc10",
    )
    .unwrap();

    let account = SingleOwnerAccount::new(provider, signer, address, chain_id::TESTNET);

    let result = account
        .execute(vec![
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("1000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("2000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
        ])
        .simulate()
        .await
        .unwrap();

    assert!(!result.trace.function_invocation.internal_calls.is_empty());
}

#[tokio::test]
async fn can_execute_tst_mint() {
    // This test case is not very useful as the sequencer will always respond with
    // `TransactionReceived` even if the transaction will eventually fail, just like how
    // `eth_sendRawTransaction` always responds with success except for insufficient balance. So it
    // can't really test the execution is successful unless we:
    //   - change to use a local testing network similar to Ganacha for Ethereum; or
    //   - poll the transaction hash until it's processed.

    let provider = SequencerGatewayProvider::starknet_alpha_goerli();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "02da37a17affbd2df4ede7120dae305ec36dfe94ec96a8c3f49bbf59f4e9a9fa",
    )
    .unwrap();
    let tst_token_address = FieldElement::from_hex_be(
        "07394cbe418daa16e42b87ba67372d4ab4a5df0b05c6e554d158458ce245bc10",
    )
    .unwrap();

    let account = SingleOwnerAccount::new(provider, signer, address, chain_id::TESTNET);

    let result = account
        .execute(vec![
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("1000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
            Call {
                to: tst_token_address,
                selector: get_selector_from_name("mint").unwrap(),
                calldata: vec![
                    address,
                    FieldElement::from_dec_str("2000000000000000000000").unwrap(),
                    FieldElement::ZERO,
                ],
            },
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(result.code, AddTransactionResultCode::TransactionReceived);
}

#[tokio::test]
async fn can_declare_oz_account_contract() {
    // This test case is not very useful, same as `can_execute_tst_mint` above.

    let provider = SequencerGatewayProvider::starknet_alpha_goerli();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "02da37a17affbd2df4ede7120dae305ec36dfe94ec96a8c3f49bbf59f4e9a9fa",
    )
    .unwrap();
    let account = SingleOwnerAccount::new(provider, signer, address, chain_id::TESTNET);

    let contract_artifact: ContractArtifact =
        serde_json::from_str(include_str!("../test-data/artifacts/oz_account.txt")).unwrap();

    let result = account
        .declare(Arc::new(contract_artifact))
        .send()
        .await
        .unwrap();

    assert_eq!(result.code, AddTransactionResultCode::TransactionReceived);
}
