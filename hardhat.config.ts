import { HardhatUserConfig, vars, task } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const LOTTO_MINATO_PRIVATE_KEY = vars.get("LOTTO_MINATO_PRIVATE_KEY");

task("accounts", "Prints the list of accounts", async (taskArgs, hre) => {
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


const config: HardhatUserConfig = {
  solidity: "0.8.24", // replace if necessary
  networks: {
    'minato': {
      url: 'https://rpc.minato.soneium.org',
      accounts: [LOTTO_MINATO_PRIVATE_KEY]
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

