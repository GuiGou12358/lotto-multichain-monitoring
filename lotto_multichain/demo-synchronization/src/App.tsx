import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "WxR8LgRfZj74duZuxEhLmFYXjmAB3DRzZuM2sd51An7NvfB";
const registration1Address ="XWfadAkwxCUXAgWsGhC6o5ckLh62AYerLjhkpyHoZf7pfMr";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0xACDCb69F2C7C1F56C693244F6F5c004A56D3e7E3";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0x45e6301df1d152C7c048EECAFA658E88fD5A5897";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0xc8950613bfd19463ca39d8508c30cbbf310091569de0edf91d429923adbf9929";

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
                           rpcCommunicatingContract={rpcPhala} addressCommunicatingContract={pinkContractAddress}/>
        </svg>
      </header>
    </div>
  );
}
