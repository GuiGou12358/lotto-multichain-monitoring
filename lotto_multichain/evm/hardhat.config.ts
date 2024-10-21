import { HardhatUserConfig, vars, task } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const MINATO_OWNER_PRIVATE_KEY = vars.get("LOTTO_MINATO_OWNER_KEY");
const MINATO_EVM_ATTESTOR_PK = vars.get("LOTTO_MINATO_ATTESTOR_KEY");
const MINATO_USER1_PRIVATE_KEY = vars.get("LOTTO_MINATO_USER1_KEY");
const MINATO_USER2_PRIVATE_KEY = vars.get("LOTTO_MINATO_USER2_KEY");

task("accounts", "Prints the list of accounts", async (_taskArgs, hre) => {
  const accounts = await hre.ethers.getSigners();

  for (const account of accounts) {
    console.log(account.address);
  }
});

task("balance", "Prints an account's balance", async (taskArgs, hre) => {
  // @ts-ignore
  const balance = await hre.ethers.provider.getBalance(taskArgs.account);
  console.log(hre.ethers.formatEther(balance), "ETH");
}).addParam("account", "The account's address");

task("balances", "Prints the balance for all accounts", async (_taskArgs, hre) => {

  const accounts = await hre.ethers.getSigners();

  for (const account of accounts) {
    const balance =  await hre.ethers.provider.getBalance(account.address);
    const formattedBalance = hre.ethers.formatEther(balance);
    console.log("%s - %s ETH", account.address, formattedBalance);
  }

});


const config: HardhatUserConfig = {
  solidity: "0.8.24", // replace if necessary
  networks: {
    'minato': {
      url: 'https://rpc.minato.soneium.org',
      accounts: [MINATO_OWNER_PRIVATE_KEY, MINATO_EVM_ATTESTOR_PK, MINATO_USER1_PRIVATE_KEY, MINATO_USER2_PRIVATE_KEY]
    },
  },
  etherscan: {
    apiKey: {
      'minato': 'empty'
    },
    customChains: [
      {
        network: "minato",
        chainId: 1946,
        urls: {
          apiURL: "https://explorer-testnet.soneium.org/api",
          browserURL: "https://explorer-testnet.soneium.org"
        }
      }
    ]
  }
};

export default config;

