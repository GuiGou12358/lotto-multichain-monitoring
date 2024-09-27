extern crate alloc;
extern crate core;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use pink_extension::{debug, http_post, info};
use scale::{Decode, Encode};
use serde::Deserialize;
use serde_json_core;
use sp_core::crypto::Ss58Codec;
use ink::prelude::{format, string::String};
use alloc::vec::Vec;

/// DTO use for serializing and deserializing the json when querying the winners
#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct IndexerParticipationsResponse<'a> {
    #[serde(borrow)]
    data: IndexerParticipationsResponseData<'a>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
struct IndexerParticipationsResponseData<'a> {
    #[serde(borrow)]
    participations: Participations<'a>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
struct Participations<'a> {
    #[serde(borrow)]
    nodes: Vec<ParticipationNode<'a>>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
struct ParticipationNode<'a> {
    accountId: &'a str,
}

/// DTO use for serializing and deserializing the json when querying the hashes
#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct IndexerHashesResponse<'a> {
    #[serde(borrow)]
    data: IndexerHashesResponseData<'a>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
struct IndexerHashesResponseData<'a> {
    #[serde(borrow)]
    endRaffles: EndRaffle<'a>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
struct EndRaffle<'a> {
    #[serde(borrow)]
    nodes: Vec<EndRaffleNode<'a>>,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
struct EndRaffleNode<'a> {
    lottoId: &'a str,
    hash: &'a str,
}

pub struct Indexer {
    endpoint: String,
}

impl Indexer{

    pub fn new(url: Option<String>) -> Result<Self, RaffleDrawError> {
        let endpoint = url.ok_or(IndexerNotConfigured)?;
        Ok(Self { endpoint })
    }

    pub fn query_winners(
        self,
        raffle_id: RaffleId,
        numbers: &Vec<Number>,
    ) -> Result<Vec<AccountId32>, RaffleDrawError> {
        info!(
                "Request received to get the winners for raffle id {raffle_id} and numbers {numbers:?} "
            );

        if numbers.is_empty() {
            return Err(NoNumber);
        }

        // build the headers
        let headers = alloc::vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into())
        ];
        // build the filter
        let mut filter = format!(
            r#"filter:{{and:[{{numRaffle:{{equalTo:\"{}\"}}}}"#,
            raffle_id
        );
        for n in numbers {
            let f = format!(r#",{{numbers:{{contains:\"{}\"}}}}"#, n);
            filter.push_str(&f);
        }
        filter.push_str("]}");

        // build the body
        let body = format!(
            r#"{{"query" : "{{participations({}){{ nodes {{ accountId }} }} }}"}}"#,
            filter
        );

        debug!("body: {body}");

        // query the indexer
        let resp = http_post!(self.endpoint, body, headers);

        // check the result
        if resp.status_code != 200 {
            ink::env::debug_println!("status code {}", resp.status_code);
            return Err(HttpRequestFailed);
        }

        // parse the result
        let result: IndexerParticipationsResponse =
            serde_json_core::from_slice(resp.body.as_slice())
                .or(Err(InvalidResponseBody))?
                .0;

        // add the winners
        let mut winners = Vec::new();
        for w in result.data.participations.nodes.iter() {
            // build the accountId from the string address
            let account_id =
                sp_core::crypto::AccountId32::from_ss58check(w.accountId).or(Err(InvalidSs58Address))?;
            let address_hex: [u8; 32] = scale::Encode::encode(&account_id)
                .try_into()
                .or(Err(InvalidKeyLength))?;
            winners.push(address_hex); // TODO manage AccountId32 and AccountId20
        }

        info!("Winners: {winners:02x?}");

        Ok(winners)
    }

    pub fn query_hashes(
        self,
        raffle_id: RaffleId
    ) -> Result<Vec<Hash>, RaffleDrawError> {
        info!("Query hashes for raffle id {raffle_id}");

        // build the headers
        let headers = alloc::vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into())
        ];
        // build the filter
        let filter = format!(r#"filter:{{numRaffle:{{equalTo:\"{}\"}}}}"#, raffle_id);

        // build the body
        let body = format!(
            r#"{{"query" : "{{endRaffle({}){{ nodes {{ lottoId, hash }} }} }}"}}"#,
            filter
        );

        debug!("body: {body}");

        // query the indexer
        let resp = http_post!(self.endpoint, body, headers);

        // check the result
        if resp.status_code != 200 {
            ink::env::debug_println!("status code {}", resp.status_code);
            return Err(HttpRequestFailed);
        }

        // parse the result
        let result: IndexerHashesResponse = serde_json_core::from_slice(resp.body.as_slice())
            .or(Err(InvalidResponseBody))?
            .0;

        // add the hashes
        let mut hashes = Vec::new();
        for node in result.data.endRaffles.nodes.iter() {
            // build the accountId from the string address
            let hash_raw: [u8; 32] = hex::decode(node.hash)
                .expect("hex decode failed")
                .try_into()
                .expect("incorrect length");
            hashes.push(hash_raw);
        }

        info!("Hashes: {hashes:02x?}");

        Ok(hashes)
    }
}
