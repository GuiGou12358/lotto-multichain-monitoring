use crate::error::RaffleError;
use crate::error::RaffleError::*;
use crate::Number;
use openbrush::traits::Storage;

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    config: Option<Config>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Config {
    pub nb_numbers: u8,
    pub min_number: Number,
    pub max_number: Number,
}

#[openbrush::trait_definition]
pub trait RaffleConfig: Storage<Data> {
    fn set_config(&mut self, config: Config) -> Result<(), RaffleError> {
        // check the config
        if config.nb_numbers == 0 {
            return Err(IncorrectConfig);
        }

        if config.min_number >= config.max_number {
            return Err(IncorrectConfig);
        }

        self.data::<Data>().config = Some(config);
        Ok(())
    }

    #[ink(message)]
    fn get_config(&self) -> Option<Config> {
        self.data::<Data>().config
    }

    /// return the config and throw an error of the config is missing
    fn ensure_config(&self) -> Result<Config, RaffleError> {
        match self.data::<Data>().config {
            None => Err(ConfigNotSet),
            Some(config) => Ok(config),
        }
    }

    /// check if the config is the same as the one given in parameter
    fn ensure_same_config(&self, config: &Config) -> Result<(), RaffleError> {
        // get the correct results for the given raffle
        let this_config = self.ensure_config()?;

        if this_config.nb_numbers != config.nb_numbers
            || this_config.min_number != config.min_number
            || this_config.max_number != config.max_number
        {
            return Err(DifferentConfig);
        }

        Ok(())
    }

    /// check if the numbers respect the config
    fn check_numbers(&mut self, numbers: &[Number]) -> Result<(), RaffleError> {
        // check if the config is set
        let config = self.ensure_config()?;

        // check the numbers
        let nb_numbers = numbers.len();

        if nb_numbers != config.nb_numbers as usize {
            return Err(IncorrectNbNumbers);
        }

        for number in numbers.iter() {
            if *number > config.max_number || *number < config.min_number {
                return Err(IncorrectNumbers);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_contract::lotto_contract::Contract;

    #[ink::test]
    fn test_bad_config() {
        let mut contract = Contract::new();

        let result = contract.set_config(Config {
            nb_numbers: 0,
            min_number: 1,
            max_number: 50,
        });
        assert_eq!(result, Err(IncorrectConfig));

        let result = contract.set_config(Config {
            nb_numbers: 0,
            min_number: 10,
            max_number: 10,
        });
        assert_eq!(result, Err(IncorrectConfig));

        let result = contract.set_config(Config {
            nb_numbers: 4,
            min_number: 51,
            max_number: 50,
        });
        assert_eq!(result, Err(IncorrectConfig));
    }

    #[ink::test]
    fn test_get_config() {
        let mut contract = Contract::new();

        let config = contract.get_config();
        assert_eq!(config, None);

        contract
            .set_config(Config {
                nb_numbers: 4,
                min_number: 1,
                max_number: 50,
            })
            .expect("failed to set the config");

        if let Some(config) = contract.get_config() {
            assert_eq!(config.nb_numbers, 4);
            assert_eq!(config.min_number, 1);
            assert_eq!(config.max_number, 50);
        } else {
            panic!("No config found")
        }
    }

    #[ink::test]
    fn test_ensure_config() {
        let mut contract = Contract::new();

        assert_eq!(contract.ensure_config(), Err(ConfigNotSet));

        contract
            .set_config(Config {
                nb_numbers: 4,
                min_number: 1,
                max_number: 50,
            })
            .expect("failed to set the config");

        let config = contract.ensure_config().expect("failed to set the config");
        assert_eq!(config.nb_numbers, 4);
        assert_eq!(config.min_number, 1);
        assert_eq!(config.max_number, 50);
    }

    #[ink::test]
    fn test_ensure_same_config() {
        let mut contract = Contract::new();

        let config = contract.get_config();
        assert_eq!(config, None);

        contract
            .set_config(Config {
                nb_numbers: 4,
                min_number: 1,
                max_number: 50,
            })
            .expect("failed to set the config");

        contract
            .ensure_same_config(&Config {
                nb_numbers: 4,
                min_number: 1,
                max_number: 50,
            })
            .expect("failed to set the config");

        let result = contract.ensure_same_config(&Config {
            nb_numbers: 5,
            min_number: 1,
            max_number: 50,
        });
        assert_eq!(result, Err(DifferentConfig));

        let result = contract.ensure_same_config(&Config {
            nb_numbers: 4,
            min_number: 0,
            max_number: 50,
        });
        assert_eq!(result, Err(DifferentConfig));

        let result = contract.ensure_same_config(&Config {
            nb_numbers: 4,
            min_number: 1,
            max_number: 51,
        });
        assert_eq!(result, Err(DifferentConfig));
    }

    #[ink::test]
    fn test_check_numbers() {
        let mut contract = Contract::new();

        let config = contract.get_config();
        assert_eq!(config, None);

        contract
            .set_config(Config {
                nb_numbers: 4,
                min_number: 1,
                max_number: 50,
            })
            .expect("failed to set the config");

        contract
            .check_numbers(vec![5u16, 2, 49, 13].as_slice())
            .expect("failed to check numbers");

        contract
            .check_numbers(vec![5u16, 9, 1, 50].as_slice())
            .expect("failed to check numbers");

        let result = contract.check_numbers(vec![9u16, 10, 25].as_slice());
        assert_eq!(result, Err(IncorrectNbNumbers));

        let result = contract.check_numbers(vec![9u16, 10, 25, 51].as_slice());
        assert_eq!(result, Err(IncorrectNumbers));

        let result = contract.check_numbers(vec![9u16, 10, 25, 0].as_slice());
        assert_eq!(result, Err(IncorrectNumbers));
    }
}
