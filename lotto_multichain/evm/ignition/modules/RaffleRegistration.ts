import {buildModule} from "@nomicfoundation/hardhat-ignition/modules";

const raffleRegistration = buildModule("RaffleRegistration", (m) => {
    const owner = m.getAccount(0);
    const lottoClient = m.contract("RaffleRegistration", [owner]);

    return { lottoClient }
});

export default raffleRegistration;