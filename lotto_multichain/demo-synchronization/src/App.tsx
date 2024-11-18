import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "YuCwFXie1QX7pPyvL8dHYGCX6gpCPB4aWchzb2bSYSgdrvu";
const registration1Address ="ZkMmwcAsCuFPB13kfXH6aQiJYAfnQYC2qMiK5h1mGKsMX86";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0x991926D5ca21EF2938B5BAffbf4EC24fB55e205e";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0xA8AE9c3F7bc784Ccd1E6013c59A233600C6dE90A";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0x49badf682da735bee55e1098414edbd993f3500893b35cadd2d610bc961b0d33";

export default function App() {
  return (
    <div className="App">
      <header className="App-header">
        <svg width="1000" height="800">
          <LegendManagerWorkflow cx={0}/>
          <ManagerWorkflow cx={250} rpc={rpcShibuya} address={managerAddress} chain={"Astar testnet"}
                           explorer={"https://shibuya.subscan.io/wasm_contract/"}
          />
          <ParticipationWorkflowWasm cx={400} rpc={rpcShibuya} address={registration1Address} chain={"Astar testnet"}
                                     explorer={"https://shibuya.subscan.io/wasm_contract/"}/>
          <ParticipationWorkflowEvm cx={500} rpc={rpcMoonbase} address={registration2Address} chain={"Moonbeam test"}
                                    explorer={"https://moonbase.moonscan.io/address/"}/>
          <ParticipationWorkflowEvm cx={600} rpc={rpcMinato} address={registration3Address} chain={"Soneium testnet"}
                                    explorer={"https://soneium-minato.blockscout.com/address/"}/>
          <LegendParticipationWorkflow cx={700}/>
          <Synchronisation rpcManagerContract={rpcShibuya} addressManagerContract={managerAddress}
                           rpcCommunicatingContract={rpcPhala} addressCommunicatingContract={pinkContractAddress}
                           explorerCommunicatingContract={"https://phala.subscan.io/wasm_contract/"}/>
        </svg>
      </header>
    </div>
  );
}
