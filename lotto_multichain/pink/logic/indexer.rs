extern crate alloc;
extern crate core;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::vec::Vec;
use ink::prelude::{format, string::String};
use pink_extension::{debug, http_post, info};
use scale::{Encode};
use serde::Deserialize;
use serde_json_core;
use sp_core::crypto::Ss58Codec;

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

impl Indexer {
    pub fn new(url: Option<String>) -> Result<Self, RaffleDrawError> {
        let endpoint = url.ok_or(IndexerNotConfigured)?;
        Ok(Self { endpoint })
    }

    pub fn query_winners(
        self,
        draw_number: DrawNumber,
        numbers: &Vec<Number>,
    ) -> Result<(Vec<AccountId32>, Vec<AccountId20>), RaffleDrawError> {
        info!(
                "Request received to get the winners for raffle id {draw_number} and numbers {numbers:?} "
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
            draw_number
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
            let account_id = sp_core::crypto::AccountId32::from_ss58check(w.accountId)
                .or(Err(InvalidSs58Address))?;
            let address_hex: [u8; 32] = scale::Encode::encode(&account_id)
                .try_into()
                .or(Err(InvalidKeyLength))?;
            winners.push(address_hex); // TODO manage AccountId32 and AccountId20
        }

        info!("Winners: {winners:02x?}");

        Ok((winners, Vec::new())) // TODO manage AccountId32 and AccountId20
    }

    pub fn query_hashes(self, draw_number: DrawNumber) -> Result<Vec<Hash>, RaffleDrawError> {
        info!("Query hashes for raffle id {draw_number}");

        // build the headers
        let headers = alloc::vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into())
        ];
        // build the filter
        let filter = format!(r#"filter:{{numRaffle:{{equalTo:\"{}\"}}}}"#, draw_number);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn new_indexer() -> Indexer {
        Indexer {
            endpoint: "https://query.substrate.fi/lotto-subquery-shibuya".to_string(),
        }
    }

    // TODO ink::test or test
    #[ink::test]
    fn test_get_winners() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let draw_num = 2;
        let numbers = vec![15, 1, 44, 28];

        let indexer = new_indexer();
        let winners = indexer.query_winners(draw_num, &numbers).unwrap();
        ink::env::debug_println!("winners: {winners:?}");
    }

    #[ink::test]
    fn test_no_winner() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let draw_num = 0;
        let numbers = vec![150, 1, 44, 2800];

        let indexer = new_indexer();
        let winners = indexer.query_winners(draw_num, &numbers).unwrap();
        assert_eq!(0, winners.0.len());
        assert_eq!(0, winners.1.len());
    }

    #[ink::test]
    fn test_no_number() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let draw_num = 0;
        let numbers = vec![];

        let indexer = new_indexer();
        let result = indexer.query_winners(draw_num, &numbers);
        assert_eq!(Err(NoNumber), result);
    }
}