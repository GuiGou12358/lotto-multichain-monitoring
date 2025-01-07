import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "acKRTkuTsGESFmsy3wcvWTyfRW6xL4QY1gMhjNVtD5yUB7d";
const registration1Address ="bSm4f7WjbxFMbo4fRUGw7oHvva65P8m8jCqedFsXAwUJx7V";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0x987461a5eF325f9f217D2b777CeDCf3b9c4D62d5";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0x04d884675E5790721cb5F24D41D460E921C08f17";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0x269bdbdef7285a1bdba7f68c79c1165705f20508e5e7c545b4fd10a6f345523e";

export default function App() {
  return (
    <div className="App">
      <header className="App-header">
        <svg width="1000" height="1000">
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
