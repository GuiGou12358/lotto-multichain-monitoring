import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "baB95cq8LN1Bzafv29M88b3hj57WiNuQ7aZz9FJDzdMhPE2";
const registration1Address ="bKFW6HcX4GE6LkAq3464EaSoQdnSy9HiMXgKhAVu3bSK1oe";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0xA50d6F87E90A28c5b594d262469c1B93c0C2f874";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0xBFaFd55C6Cc933174308B461C0b1DD5379cc7152";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0x9ded0fa7da38f03a7fd6128f92055afb926176bea254a9e29ccbc1acdc649318";

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
