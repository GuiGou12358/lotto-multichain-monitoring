const LottoClient = artifacts.require("LottoClient");

module.exports = function(deployer, network, accounts) {
  const addressOne = accounts[0];
  deployer.deploy(LottoClient, addressOne);
};
