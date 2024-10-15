use ink::env::{debug_println, DefaultEnvironment};
use ink_e2e::subxt::tx::Signer;
use ink_e2e::{build_message, PolkadotConfig};
use openbrush::contracts::access_control::accesscontrol_external::AccessControl;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use scale::Decode;
use scale::Encode;

use lotto::config::Config;
use lotto::*;

use lotto::raffle_registration::raffle_external::Raffle;
use lotto_registration_contract::{lotto_registration_contract, *};

use phat_rollup_anchor_ink::traits::meta_transaction::metatransaction_external::MetaTransaction;
use phat_rollup_anchor_ink::traits::rollup_anchor::rollupanchor_external::RollupAnchor;

use phat_rollup_anchor_ink::traits::rollup_anchor::*;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn alice_instantiates_raffle_registration(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
) -> AccountId {
    let lotto_constructor = lotto_registration_contract::ContractRef::new();
    let contract_id = client
        .instantiate(
            "lotto_registration_contract",
            &ink_e2e::alice(),
            lotto_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;

    contract_id
}

async fn alice_grants_bob_as_attestor(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) {
    // bob is granted as attestor
    let bob_address = ink::primitives::AccountId::from(ink_e2e::bob().public_key().0);
    let grant_role = build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
        .call(|contract| contract.grant_role(ATTESTOR_ROLE, Some(bob_address)));
    client
        .call(&ink_e2e::alice(), grant_role, 0, None)
        .await
        .expect("grant bob as attestor failed");
}

async fn attestor_set_config_and_start(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    config: Config,
) {
    let payload = RequestForAction::SetConfigAndStart(config.clone());

    let actions = vec![HandleActionInput::Reply(payload.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("set config failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));

    // check the status
    assert_eq!(
        raffle_registration::Status::Started,
        get_status(client, contract_id).await
    );

}

async fn attestor_open_registrations(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
) {
    let payload = RequestForAction::OpenRegistrations(draw_number);

    let actions = vec![HandleActionInput::Reply(payload.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    /*
              let result = client.call_dry_run(&ink_e2e::bob(), &rollup_cond_eq, 0, None).await;
              assert_eq!(
                  result.debug_message(),
                  "Debug message"
              );
    */

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("open registrations failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));

    // check the draw number and the status
    assert_eq!(draw_number, get_draw_number(client, contract_id).await);
    assert_eq!(
        raffle_registration::Status::RegistrationsOpen,
        get_status(client, contract_id).await
    );
}

async fn attestor_close_registrations(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
) {
    let payload = RequestForAction::CloseRegistrations(draw_number);

    let actions = vec![HandleActionInput::Reply(payload.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("close registrations failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));

    // check the draw number and the status
    assert_eq!(draw_number, get_draw_number(client, contract_id).await);
    assert_eq!(
        raffle_registration::Status::RegistrationsClosed,
        get_status(client, contract_id).await
    );
}

async fn attestor_set_results(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    numbers: Vec<Number>,
    winners: Vec<AccountId>,
) {
    let payload = RequestForAction::SetResults(draw_number, numbers.clone(), winners.clone());

    let actions = vec![HandleActionInput::Reply(payload.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("Set results failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));

    // check the draw number and the status
    assert_eq!(draw_number, get_draw_number(client, contract_id).await);
    assert_eq!(
        raffle_registration::Status::ResultsReceived,
        get_status(client, contract_id).await
    );
}

async fn participates(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    signer: &ink_e2e::Keypair,
    numbers: Vec<Number>,
) {
    let participate =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.participate(numbers.clone()));
    client
        .call(signer, participate, 0, None)
        .await
        .expect("Participate failed");
}

async fn can_participate(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> bool {
    let can_participate =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.can_participate());

    let result = client
        .call_dry_run(&ink_e2e::alice(), &can_participate, 0, None)
        .await
        .return_value();

    result
}

async fn get_draw_number(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> DrawNumber {
    let get_draw_number =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_draw_number());

    let draw_number = client
        .call_dry_run(&ink_e2e::alice(), &get_draw_number, 0, None)
        .await
        .return_value();

    draw_number
}
/*
   async fn get_last_raffle_for_verif(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
   ) -> Option<RaffleId> {
       // check in the kv store
       const LAST_RAFFLE_FOR_VERIF: u32 = ink::selector_id!("LAST_RAFFLE_FOR_VERIF");

       let get_value = build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
           .call(|contract| contract.get_value(LAST_RAFFLE_FOR_VERIF.encode()));

       let raffle_id = client
           .call_dry_run(&ink_e2e::alice(), &get_value, 0, None)
           .await
           .return_value();

       match raffle_id {
           Some(r) => Some(
               RaffleId::decode(&mut r.as_slice()).expect("Cannot decode Last raffle for verif"),
           ),
           None => None,
       }
   }
*/

async fn get_status(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> raffle_registration::Status {
    let get_status = build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
        .call(|contract| contract.get_status());

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_status, 0, None)
        .await;

    result.return_value()
}

#[ink_e2e::test(
    additional_contracts = "contracts/raffle_registration/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_raffles(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_registration(&mut client).await;

    let config = Config {
        nb_numbers: 4,
        min_number: 1,
        max_number: 50,
    };

    // bob is granted as attestor
    alice_grants_bob_as_attestor(&mut client, &contract_id).await;

    assert_eq!(0, get_draw_number(&mut client, &contract_id).await);
    assert_eq!(
        raffle_registration::Status::NotStarted,
        get_status(&mut client, &contract_id).await
    );

    // configure the raffle
    attestor_set_config_and_start(&mut client, &contract_id, config.clone()).await;

    // check if the user can participate
    assert_eq!(false, can_participate(&mut client, &contract_id).await);

    // Open the registrations
    attestor_open_registrations(&mut client, &contract_id, 10).await;

    // check if the user can participate
    assert_eq!(true, can_participate(&mut client, &contract_id).await);

    // dave participates
    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![5, 40, 8, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![3, 6, 7, 5],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![12, 4, 6, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![15, 44, 4, 1],
    )
    .await;

    // charlie participates
    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![50, 3, 8, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![34, 6, 2, 5],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![12, 4, 6, 4],
    )
    .await;

    // Close the registrations
    attestor_close_registrations(&mut client, &contract_id, 10).await;

    // check if the user can participate
    assert_eq!(false, can_participate(&mut client, &contract_id).await);

    // Set the results
    let numbers = vec![5, 6, 7, 8];
    let winners = vec![];
    attestor_set_results(&mut client, &contract_id, 10, numbers, winners).await;

    // check if the user can participate
    assert_eq!(false, can_participate(&mut client, &contract_id).await);

    // Open again the registrations
    attestor_open_registrations(&mut client, &contract_id, 11).await;

    // check if the user can participate
    assert_eq!(true, can_participate(&mut client, &contract_id).await);

    // dave participates
    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![5, 40, 8, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![3, 6, 7, 5],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![12, 4, 6, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::dave(),
        vec![15, 44, 4, 1],
    )
    .await;

    // charlie participates
    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![50, 3, 8, 2],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![34, 6, 2, 5],
    )
    .await;

    participates(
        &mut client,
        &contract_id,
        &ink_e2e::charlie(),
        vec![12, 4, 6, 4],
    )
    .await;

    // Close the registrations
    attestor_close_registrations(&mut client, &contract_id, 11).await;

    // Set the results
    let numbers = vec![12, 4, 6, 4];
    let charlie_address = ink::primitives::AccountId::from(ink_e2e::charlie().public_key().0);
    let winners = vec![charlie_address];
    attestor_set_results(&mut client, &contract_id, 11, numbers, winners).await;

    Ok(())
}

#[ink_e2e::test(
    additional_contracts = "contracts/raffle_registration/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_bad_attestor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_registration(&mut client).await;

    // bob is not granted as attestor => it should not be able to send a message
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], vec![]));
    let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
    assert!(
        result.is_err(),
        "only attestor should be able to send messages"
    );

    // bob is granted as attestor
    alice_grants_bob_as_attestor(&mut client, &contract_id).await;

    // then bob is able to send a message
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], vec![]));
    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("rollup cond eq failed");
    // no event
    assert!(!result.contains_event("Contracts", "ContractEmitted"));

    Ok(())
}

#[ink_e2e::test(
    additional_contracts = "contracts/raffle_registration/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_registration(&mut client).await;

    // bob is granted as attestor
    alice_grants_bob_as_attestor(&mut client, &contract_id).await;

    let actions = vec![HandleActionInput::Reply(58u128.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));
    let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
    assert!(
        result.is_err(),
        "we should not be able to proceed bad messages"
    );

    Ok(())
}

///
/// Test the meta transactions
/// Alice is the owner
/// Bob is the attestor
/// Charlie is the sender (ie the payer)
///
#[ink_e2e::test(
    additional_contracts = "contracts/raffle_registration/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_meta_tx_rollup_cond_eq(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    let contract_id = alice_instantiates_raffle_registration(&mut client).await;

    // Bob is the attestor
    // use the ecsda account because we are not able to verify the sr25519 signature
    let from = ink::primitives::AccountId::from(
        Signer::<PolkadotConfig>::account_id(&subxt_signer::ecdsa::dev::bob()).0,
    );

    // add the role => it should be succeed
    let grant_role = build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
        .call(|contract| contract.grant_role(ATTESTOR_ROLE, Some(from)));
    client
        .call(&ink_e2e::alice(), grant_role, 0, None)
        .await
        .expect("grant the attestor failed");

    // prepare the meta transaction
    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
    let prepare_meta_tx =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.prepare(from, data.clone()));
    let result = client
        .call(&ink_e2e::bob(), prepare_meta_tx, 0, None)
        .await
        .expect("We should be able to prepare the meta tx");

    let (request, _hash) = result
        .return_value()
        .expect("Expected value when preparing meta tx");

    assert_eq!(0, request.nonce);
    assert_eq!(from, request.from);
    assert_eq!(contract_id, request.to);
    assert_eq!(&data, &request.data);

    // Bob signs the message
    let keypair = subxt_signer::ecdsa::dev::bob();
    let signature = keypair.sign(&scale::Encode::encode(&request)).0;

    // do the meta tx: charlie sends the message
    let meta_tx_rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.meta_tx_rollup_cond_eq(request.clone(), signature));
    client
        .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
        .await
        .expect("meta tx rollup cond eq should not failed");

    // do it again => it must fail
    let meta_tx_rollup_cond_eq =
        build_message::<lotto_registration_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.meta_tx_rollup_cond_eq(request.clone(), signature));
    let result = client
        .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
        .await;
    assert!(
        result.is_err(),
        "This message should not be proceed because the nonce is obsolete"
    );

    Ok(())
}
