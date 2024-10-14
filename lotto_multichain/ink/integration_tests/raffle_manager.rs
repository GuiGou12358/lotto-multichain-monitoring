use ink::env::{debug_println, DefaultEnvironment};
use ink_e2e::subxt::tx::Signer;
use ink_e2e::{build_message, PolkadotConfig};
use openbrush::contracts::access_control::accesscontrol_external::AccessControl;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use scale::Decode;
use scale::Encode;

use lotto::config::Config;
use lotto::raffle_manager;
use lotto::*;

use lotto::raffle_manager::rafflemanager_external::RaffleManager;
use lotto_registration_manager_contract::{lotto_registration_manager_contract, *};

use phat_rollup_anchor_ink::traits::meta_transaction::metatransaction_external::MetaTransaction;
use phat_rollup_anchor_ink::traits::rollup_anchor::rollupanchor_external::RollupAnchor;

use phat_rollup_anchor_ink::traits::rollup_anchor::*;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn alice_instantiates_raffle_manager(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
) -> AccountId {
    let lotto_constructor = lotto_registration_manager_contract::ContractRef::new();
    let lotto_contract_id = client
        .instantiate(
            "lotto_registration_manager_contract",
            &ink_e2e::alice(),
            lotto_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;

    lotto_contract_id
}

async fn alice_configures_raffle_manager(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    config: Config,
    registration_contracts: Vec<RegistrationContractId>,
) {
    let set_config =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.set_config(config));
    client
        .call(&ink_e2e::alice(), set_config, 0, None)
        .await
        .expect("set config failed");

    for registration_contract_id in registration_contracts {
        let add_registration_contract =
            build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
                .call(|contract| contract.add_registration_contract(registration_contract_id));

        client
            .call(&ink_e2e::alice(), add_registration_contract, 0, None)
            .await
            .expect("add registration contract failed");
    }
}

async fn alice_grants_bob_as_attestor(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) {
    // bob is granted as attestor
    let bob_address = ink::primitives::AccountId::from(ink_e2e::bob().public_key().0);
    let grant_role =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.grant_role(ATTESTOR_ROLE, Some(bob_address)));
    client
        .call(&ink_e2e::alice(), grant_role, 0, None)
        .await
        .expect("grant bob as attestor failed");
}

async fn alice_starts_raffle(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    previous_draw_number: DrawNumber,
) {
    let start_raffle =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.start(previous_draw_number));
    client
        .call(&ink_e2e::alice(), start_raffle, 0, None)
        .await
        .expect("start raffle failed");
}

async fn bob_sends_config_propagated(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    registration_contracts: Vec<RegistrationContractId>,
    queue_head: u32,
) {
    let hash = [0u8; 32];
    let payload =
        LottoManagerResponseMessage::ConfigPropagated(registration_contracts.clone(), hash.into());

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    /*
           let result = client.call_dry_run(&ink_e2e::bob(), &rollup_cond_eq, 0, None).await;
           assert_eq!(
               result.debug_message(),
               "only attestor should be able to send messages"
           );
    */

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send config propagated failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));
}

async fn bob_sends_all_registrations_open(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    registration_contracts: Vec<RegistrationContractId>,
    queue_head: u32,
) {
    let payload =
        LottoManagerResponseMessage::RegistrationsOpen(draw_number, registration_contracts.clone());

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send config propagated failed");
    // two events : MessageProcessedTo and RaffleDone
    //assert!(result.contains_event("Contracts", "ContractEmitted"));
}

async fn alice_close_registrations(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) {
    let stop_raffle =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.close_registrations());
    client
        .call(&ink_e2e::alice(), stop_raffle, 0, None)
        .await
        .expect("stop raffle failed");
}

async fn bob_sends_all_registrations_closed(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    registration_contracts: Vec<RegistrationContractId>,
    queue_head: u32,
) {
    let payload = LottoManagerResponseMessage::RegistrationsClosed(
        draw_number,
        registration_contracts.clone(),
    );

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send config propagated failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));
}

async fn bob_sends_winning_numbers(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    numbers: Vec<Number>,
    queue_head: u32,
) {
    let hash = [0u8; 32];
    let payload =
        LottoManagerResponseMessage::WinningNumbers(draw_number, numbers.clone(), hash.into());

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send result failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));
}

async fn bob_sends_winners(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    //numbers: Vec<Number>,
    winners: Vec<AccountId>,
    queue_head: u32,
) {
    //let request = LottoManagerRequestMessage::CheckWinners(draw_number, numbers.clone());
    let hash = [0u8; 32];
    let payload = LottoManagerResponseMessage::Winners(draw_number, winners.clone(), hash.into());

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send winners failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));
}

async fn bob_sends_results_propagated(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
    registration_contracts: Vec<RegistrationContractId>,
    queue_head: u32,
) {
    let hash = [0u8; 32];
    let payload = LottoManagerResponseMessage::ResultsPropagated(
        draw_number,
        registration_contracts.clone(),
        hash.into(),
    );

    let actions = vec![
        HandleActionInput::Reply(payload.encode()),
        HandleActionInput::SetQueueHead(queue_head),
    ];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.rollup_cond_eq(vec![], vec![], actions.clone()));

    /*
    let result = client
        .call_dry_run(&ink_e2e::bob(), &rollup_cond_eq, 0, None)
        .await;
    assert_eq!(
        result.debug_message(),
        "only attestor should be able to send messages"
    );
     */
    let result = client
        .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
        .await
        .expect("send results propagated failed");
    // two events : MessageProcessedTo and RaffleDone
    assert!(result.contains_event("Contracts", "ContractEmitted"));
}

/*
   async fn participates(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
       signer: &ink_e2e::Keypair,
       numbers: Vec<Number>,
   ) {
       let participate = build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
           .call(|contract| contract.participate(numbers.clone()));
       client
           .call(signer, participate, 0, None)
           .await
           .expect("Participate failed");
   }

*/

async fn get_draw_number(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> DrawNumber {
    let get_draw_number =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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

       let get_value = build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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

async fn get_manager_status(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> raffle_manager::Status {
    let get_status =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_status());

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_status, 0, None)
        .await;

    result.return_value()
}

async fn get_results(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
) -> Option<Vec<Number>> {
    let get_results =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_results(draw_number));

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_results, 0, None)
        .await;

    result.return_value()
}

async fn get_winners(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
    draw_number: DrawNumber,
) -> Option<Vec<AccountId>> {
    let get_winners =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_winners(draw_number));

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_winners, 0, None)
        .await;

    result.return_value()
}

async fn get_messages_in_queue(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    contract_id: &AccountId,
) -> Vec<LottoManagerRequestMessage> {
    const QUEUE_PREFIX: &[u8] = b"q/";
    const QUEUE_HEAD_KEY: &[u8] = b"_head";
    const QUEUE_TAIL_KEY: &[u8] = b"_tail";

    let get_queue_head =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_value([QUEUE_PREFIX, QUEUE_HEAD_KEY].concat()));

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_queue_head, 0, None)
        .await;

    let queue_head: Option<Vec<u8>> = result.return_value();
    let queue_head = match queue_head {
        Some(v) => u32::decode(&mut v.as_slice()).ok().unwrap_or_default(),
        None => 0,
    };

    let get_queue_tail =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_value([QUEUE_PREFIX, QUEUE_TAIL_KEY].concat()));

    let result = client
        .call_dry_run(&ink_e2e::alice(), &get_queue_tail, 0, None)
        .await;

    let queue_tail: Option<Vec<u8>> = result.return_value();
    let queue_tail = match queue_tail {
        Some(v) => u32::decode(&mut v.as_slice()).ok().unwrap_or_default(),
        None => 0,
    };

    let mut messages = Vec::new();
    for i in queue_head..queue_tail {
        let get_message =
            build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
                .call(|contract| contract.get_value([QUEUE_PREFIX, &i.encode()].concat()));

        let result = client
            .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
            .await;

        let message: Option<Vec<u8>> = result.return_value();
        if let Some(v) = message {
            if let Some(m) = LottoManagerRequestMessage::decode(&mut v.as_slice()).ok() {
                messages.push(m);
            }
        };
    }
    messages
}

/*
   async fn get_pending_rewards_from(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
       account_id: &AccountId,
   ) -> Option<Balance> {
       let get_pending_rewards_from =
           build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
               .call(|contract| contract.get_pending_rewards_from(*account_id));

       let result = client
           .call_dry_run(&ink_e2e::alice(), &get_pending_rewards_from, 0, None)
           .await;

       result.return_value()
   }

   async fn get_total_pending_rewards(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
   ) -> Balance {
       let get_total_pending_rewards =
           build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
               .call(|contract| contract.get_total_pending_rewards());

       let result = client
           .call_dry_run(&ink_e2e::alice(), &get_total_pending_rewards, 0, None)
           .await;

       result.return_value()
   }

   async fn fund(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
       value: Balance,
   ) {
       // check the balance of the contract
       let contract_balance_before = client
           .balance(*contract_id)
           .await
           .expect("getting contract balance failed");

       // fund the contract
       let fund_contract = build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
           .call(|contract| contract.fund());
       client
           .call(&ink_e2e::alice(), fund_contract, value, None)
           .await
           .expect("fund contract failed");

       // check the balance of the contract
       let contract_balance_after = client
           .balance(*contract_id)
           .await
           .expect("getting contract balance failed");

       assert_eq!(contract_balance_before + value, contract_balance_after);
   }

   async fn check_and_claim_rewards(
       client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
       contract_id: &AccountId,
       signer: &ink_e2e::Keypair,
       value: Balance,
   ) {
       let address = ink::primitives::AccountId::from(signer.public_key().0);

       // check the balance before claiming
       let balance_before = client
           .balance(address)
           .await
           .expect("getting balance failed");

       // check the pending rewards
       assert_eq!(
           Some(value),
           get_pending_rewards_from(client, &contract_id, &address).await
       );

       // claim the rewards
       let claim_rewards = build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
           .call(|contract| contract.claim());

       client
           .call(signer, claim_rewards, 0, None)
           .await
           .expect("claim rewards failed");
       // check the balance after claiming
       let balance_after = client
           .balance(address)
           .await
           .expect("getting balance failed");

       // we need to deduce teh fees
       //assert_eq!(balance_before + value, balance_after);
       assert!(balance_after > balance_before);

       assert_eq!(
           None,
           get_pending_rewards_from(client, &contract_id, &address).await
       );
   }
*/

#[ink_e2e::test(
    additional_contracts = "contracts/raffle_manager/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_raffles(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_manager(&mut client).await;

    let registration_contracts = vec![101, 102, 103];

    let config = Config {
        nb_numbers: 4,
        min_number: 1,
        max_number: 50,
    };

    // configure the raffle
    alice_configures_raffle_manager(
        &mut client,
        &contract_id,
        config.clone(),
        registration_contracts.clone(),
    )
    .await;

    // bob is granted as attestor
    alice_grants_bob_as_attestor(&mut client, &contract_id).await;

    assert_eq!(0, get_draw_number(&mut client, &contract_id).await);
    assert_eq!(
        raffle_manager::Status::NotStarted,
        get_manager_status(&mut client, &contract_id).await
    );

    // start the raffle
    alice_starts_raffle(&mut client, &contract_id, 10).await;

    let draw_number = get_draw_number(&mut client, &contract_id).await;
    assert_eq!(draw_number, 10);
    assert_eq!(
        raffle_manager::Status::Started,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::PropagateConfig(config.clone(), vec![101, 102, 103])
    );

    let mut queue_head = 1;

    // propagate the config
    bob_sends_config_propagated(&mut client, &contract_id, vec![], queue_head).await;
    queue_head += 1;

    // the registrations are not open because all contracts are not synched
    assert_eq!(10, get_draw_number(&mut client, &contract_id).await);
    assert_eq!(
        raffle_manager::Status::Started,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::PropagateConfig(config.clone(), vec![101, 102, 103])
    );

    // propagate the missing config
    bob_sends_config_propagated(&mut client, &contract_id, vec![101, 103], queue_head).await;
    queue_head += 1;

    // the registrations are not open because all contracts are not synched
    assert_eq!(10, get_draw_number(&mut client, &contract_id).await);
    assert_eq!(
        raffle_manager::Status::Started,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::PropagateConfig(config.clone(), vec![102])
    );

    // propagate the missing config
    bob_sends_config_propagated(&mut client, &contract_id, vec![102], queue_head).await;
    queue_head += 1;

    // the registrations are now open
    let draw_number = get_draw_number(&mut client, &contract_id).await;
    assert_eq!(draw_number, 11);
    assert_eq!(
        raffle_manager::Status::RegistrationsOpen,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the messages in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::OpenRegistrations(draw_number, vec![101, 102, 103])
    );

    // propagate registrations are open
    bob_sends_all_registrations_open(&mut client, &contract_id, draw_number, vec![], queue_head)
        .await;
    queue_head += 1;

    // all contracts are not synched
    // check the messages in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::OpenRegistrations(draw_number, vec![101, 102, 103])
    );

    // propagate all registrations are open
    bob_sends_all_registrations_open(
        &mut client,
        &contract_id,
        draw_number,
        vec![101, 102, 103],
        queue_head,
    )
    .await;
    queue_head += 1;

    // all contracts are synched
    // check the messages in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 0);

    assert_eq!(
        raffle_manager::Status::RegistrationsOpen,
        get_manager_status(&mut client, &contract_id).await
    );

    // TODO participate

    // stop the registrations
    alice_close_registrations(&mut client, &contract_id).await;
    assert_eq!(
        raffle_manager::Status::RegistrationsClosed,
        get_manager_status(&mut client, &contract_id).await
    );

    // propagate registrations are closed
    bob_sends_all_registrations_closed(
        &mut client,
        &contract_id,
        draw_number,
        vec![103],
        queue_head,
    )
    .await;
    queue_head += 1;

    // all contracts are not synched
    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::CloseRegistrations(draw_number, vec![101, 102])
    );

    // propagate registrations are closed
    bob_sends_all_registrations_closed(
        &mut client,
        &contract_id,
        draw_number,
        vec![101, 102],
        queue_head,
    )
    .await;
    queue_head += 1;

    // all contracts are synched, send the results
    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::DrawNumbers(
            draw_number,
            config.nb_numbers,
            config.min_number,
            config.max_number
        )
    );

    let numbers: Vec<Number> = vec![5, 40, 8, 2];
    bob_sends_winning_numbers(
        &mut client,
        &contract_id,
        draw_number,
        numbers.clone(),
        queue_head,
    )
    .await;
    queue_head += 1;

    assert_eq!(
        raffle_manager::Status::WaitingWinners,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::CheckWinners(draw_number, numbers.clone())
    );

    // send no winner
    let winners: Vec<AccountId> = vec![];
    bob_sends_winners(
        &mut client,
        &contract_id,
        draw_number,
        winners.clone(),
        queue_head,
    )
    .await;
    queue_head += 1;

    assert_eq!(
        raffle_manager::Status::Closed,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::PropagateResults(
            draw_number,
            numbers.clone(),
            winners.clone(),
            vec![101, 102, 103]
        )
    );

    // propagate the results
    bob_sends_results_propagated(&mut client, &contract_id, draw_number, vec![], queue_head).await;
    queue_head += 1;

    // all contracts are not synched
    // check the message in the queue
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::PropagateResults(
            draw_number,
            numbers.clone(),
            winners.clone(),
            vec![101, 102, 103]
        )
    );

    // propagate the results
    bob_sends_results_propagated(
        &mut client,
        &contract_id,
        draw_number,
        vec![101, 102, 103],
        queue_head,
    )
    .await;
    queue_head += 1;

    // all contracts are synched
    // new draw number
    let draw_number = get_draw_number(&mut client, &contract_id).await;
    assert_eq!(draw_number, 12);
    assert_eq!(
        raffle_manager::Status::RegistrationsOpen,
        get_manager_status(&mut client, &contract_id).await
    );

    // check the message in the queue
    // the registrations are opened again
    let messages = get_messages_in_queue(&mut client, &contract_id).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        LottoManagerRequestMessage::OpenRegistrations(draw_number, vec![101, 102, 103])
    );

    /*
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

    */

    Ok(())
}

#[ink_e2e::test(
    additional_contracts = "contracts/raffle_manager/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_bad_attestor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_manager(&mut client).await;

    // bob is not granted as attestor => it should not be able to send a message
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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
    additional_contracts = "contracts/raffle_manager/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // given
    let contract_id = alice_instantiates_raffle_manager(&mut client).await;

    // bob is granted as attestor
    alice_grants_bob_as_attestor(&mut client, &contract_id).await;

    let actions = vec![HandleActionInput::Reply(58u128.encode())];
    let rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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
    additional_contracts = "contracts/raffle_manager/Cargo.toml contracts/raffle_registration/Cargo.toml"
)]
async fn test_meta_tx_rollup_cond_eq(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    let contract_id = alice_instantiates_raffle_manager(&mut client).await;

    // Bob is the attestor
    // use the ecsda account because we are not able to verify the sr25519 signature
    let from = ink::primitives::AccountId::from(
        Signer::<PolkadotConfig>::account_id(&subxt_signer::ecdsa::dev::bob()).0,
    );

    // add the role => it should be succeed
    let grant_role =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.grant_role(ATTESTOR_ROLE, Some(from)));
    client
        .call(&ink_e2e::alice(), grant_role, 0, None)
        .await
        .expect("grant the attestor failed");

    // prepare the meta transaction
    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
    let prepare_meta_tx =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.meta_tx_rollup_cond_eq(request.clone(), signature));
    client
        .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
        .await
        .expect("meta tx rollup cond eq should not failed");

    // do it again => it must fail
    let meta_tx_rollup_cond_eq =
        build_message::<lotto_registration_manager_contract::ContractRef>(contract_id.clone())
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
