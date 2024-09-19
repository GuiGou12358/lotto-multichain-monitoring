import {buildModule} from "@nomicfoundation/hardhat-ignition/modules";

const lottoClientModule = buildModule("LottoClientModule", (m) => {
    const owner = m.getAccount(0);
    const lottoClient = m.contract("LottoClient", [owner]);

    return { lottoClient }
});

export default lottoClientModule;