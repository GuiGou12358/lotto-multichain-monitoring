extern crate alloc;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::vec::Vec;
use pink_extension::{info, vrf};

#[derive(scale::Encode)]
struct SaltVrf {
    contract_id: WasmContractId,
    draw_number: DrawNumber,
    hashes: Vec<Hash>,
}

pub struct Draw {
    nb_numbers: u8,
    smallest_number: Number,
    biggest_number: Number,
}

impl Draw {
    pub fn new(
        nb_numbers: u8,
        smallest_number: Number,
        biggest_number: Number,
    ) -> Result<Self, RaffleDrawError> {
        if nb_numbers == 0 {
            return Err(RaffleConfigInvalid);
        }

        if smallest_number > biggest_number {
            return Err(MinGreaterThanMax);
        }
        Ok(Self {
            nb_numbers,
            smallest_number,
            biggest_number,
        })
    }

    pub fn verify_numbers(
        &self,
        contract_id: WasmContractId,
        draw_number: DrawNumber,
        hashes: Vec<Hash>,
        numbers: Vec<Number>,
    ) -> Result<bool, RaffleDrawError> {
        let winning_numbers = self.get_numbers(contract_id, draw_number, hashes)?;
        if winning_numbers.len() != numbers.len() {
            return Ok(false);
        }

        for n in &numbers {
            if !winning_numbers.contains(n) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn get_numbers(
        &self,
        contract_id: WasmContractId,
        draw_number: DrawNumber,
        hashes: Vec<Hash>,
    ) -> Result<Vec<Number>, RaffleDrawError> {
        use ink::env::hash;

        let mut numbers = Vec::new();
        let mut i: u8 = 0;

        let salt = SaltVrf {
            contract_id,
            draw_number,
            hashes,
        };

        let encoded_salt = scale::Encode::encode(&salt);
        let mut salt_hash = <hash::Blake2x256 as hash::HashOutput>::Type::default();
        ink::env::hash_bytes::<hash::Blake2x256>(&encoded_salt, &mut salt_hash);

        while numbers.len() < self.nb_numbers as usize {
            // build a salt for this lotto_draw number
            let mut salt: Vec<u8> = Vec::new();
            salt.extend_from_slice(&i.to_be_bytes());
            salt.extend_from_slice(&salt_hash); // TODO maybe include i in hash salt

            // lotto_draw the number
            let number = self.get_number(salt, self.smallest_number, self.biggest_number)?;
            // check if the number has already been drawn
            if !numbers.iter().any(|&n| n == number) {
                // the number has not been drawn yet => we added it
                numbers.push(number);
            }
            //i += 1;
            i = i.checked_add(1).ok_or(AddOverFlow)?;
        }

        info!("Numbers: {numbers:?}");

        Ok(numbers)
    }

    fn get_number(
        &self,
        salt: Vec<u8>,
        min: Number,
        max: Number,
    ) -> Result<Number, RaffleDrawError> {
        let output = vrf(&salt);
        // keep only 8 bytes to compute the random u6²
        let mut arr = [0x00; 8];
        arr.copy_from_slice(&output[0..8]);
        let rand_u64 = u64::from_le_bytes(arr);

        // r = rand_u64() % (max - min + 1) + min
        // use u128 because (max - min + 1) can be equal to (U64::MAX - 0 + 1)
        let a = (max as u128)
            .checked_sub(min as u128)
            .ok_or(SubOverFlow)?
            .checked_add(1u128)
            .ok_or(AddOverFlow)?;
        //let b = (rand_u64 as u128) % a;
        let b = (rand_u64 as u128).checked_rem_euclid(a).ok_or(DivByZero)?;
        let r = b.checked_add(min as u128).ok_or(AddOverFlow)?;

        Ok(r as Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn test_get_numbers() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let nb_numbers = 5;
        let smallest_number = 1;
        let biggest_number = 50;
        let contract_id = [1; 32];
        let draw_number = 1;
        let hashes = vec![];

        let draw =
            Draw::new(nb_numbers, smallest_number, biggest_number).expect("Fail to init the draw");

        let result = draw.get_numbers(contract_id, draw_number, hashes).unwrap();
        assert_eq!(nb_numbers as usize, result.len());
        for &n in result.iter() {
            assert!(n >= smallest_number);
            assert!(n <= biggest_number);
        }

        ink::env::debug_println!("random numbers: {result:?}");
    }

    #[ink::test]
    fn test_get_numbers_from_1_to_5() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let nb_numbers = 5;
        let smallest_number = 1;
        let biggest_number = 5;
        let contract_id = [1; 32];
        let draw_number = 1;
        let hashes = vec![];

        let draw =
            Draw::new(nb_numbers, smallest_number, biggest_number).expect("Fail to init the draw");

        let result = draw.get_numbers(contract_id, draw_number, hashes).unwrap();
        assert_eq!(nb_numbers as usize, result.len());
        for &n in result.iter() {
            assert!(n >= smallest_number);
            assert!(n <= biggest_number);
        }

        ink::env::debug_println!("random numbers: {result:?}");
    }

    #[ink::test]
    fn test_with_different_draw_num() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let contract_id = [1; 32];
        let nb_numbers = 5;
        let smallest_number = 1;
        let biggest_number = 50;
        let hashes = vec![];

        let mut results = Vec::new();

        for i in 0..100 {
            let draw = Draw::new(nb_numbers, smallest_number, biggest_number)
                .expect("Fail to init the draw");
            let result = draw.get_numbers(contract_id, i, hashes.clone()).unwrap();
            // this result must be different from the previous ones
            results.iter().for_each(|r| assert_ne!(result, *r));

            // same request message means same result
            let result_2 = draw.get_numbers(contract_id, i, hashes.clone()).unwrap();
            assert_eq!(result, result_2);

            results.push(result);
        }
    }

    #[ink::test]
    fn test_verify_numbers() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let nb_numbers = 5;
        let smallest_number = 1;
        let biggest_number = 50;
        let contract_id = [1; 32];
        let draw_number = 1;
        let hashes = vec![];

        let draw =
            Draw::new(nb_numbers, smallest_number, biggest_number).expect("Fail to init the draw");

        let numbers = draw.get_numbers(contract_id, draw_number, hashes.clone()).unwrap();

        assert_eq!(
            Ok(true),
            draw.verify_numbers(contract_id, draw_number, hashes.clone(), numbers.clone())
        );

        // other raffle id
        assert_eq!(
            Ok(false),
            draw.verify_numbers(contract_id, draw_number + 1, hashes, numbers.clone())
        );
    }
}
