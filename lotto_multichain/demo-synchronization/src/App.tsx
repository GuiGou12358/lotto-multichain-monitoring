import './App.css';
import {CloseParticipations, LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "XFd4chL4urinkgMmAiTq38cmSa6QdbLJw1hxjghi6WcdgzY";
const registration1Address ="Z3idfVQaj7cV2sdM9Vw2mjJ1Vr5fFPm5RWECQBUQeEHV2L8";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0x22bcC50980B8C6bA38DB0E7077B4EB47dd24E351";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0x83121dDd37aa589C391b5e44bD8f94D978970bBA";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0xa59d4b80ad64e34e053575e5ae0f7664f3d0ba3c04e6ea52cb297219f3702a3c";

export default function App() {
  return (
    <div className="App">
      <header className="App-header">
        <svg width="1000" height="700">
          <LegendManagerWorkflow cx={0}/>
          <ManagerWorkflow cx={250} rpc={rpcShibuya} address={managerAddress} chain={"Astar testnet"}
                           explorer={"https://shibuya.subscan.io/wasm_contract/"}
                           rpcPinkContract={rpcPhala} addressPinkContract={pinkContractAddress}
          />
          <ParticipationWorkflowWasm cx={400} rpc={rpcShibuya} address={registration1Address} chain={"Astar testnet"}
                                     explorer={"https://shibuya.subscan.io/wasm_contract/"}/>
          <ParticipationWorkflowEvm cx={500} rpc={rpcMoonbase} address={registration2Address} chain={"Moonbeam test"}
                                    explorer={"https://moonbase.moonscan.io/address/"}/>
          <ParticipationWorkflowEvm cx={600} rpc={rpcMinato} address={registration3Address} chain={"Soneium testnet"}
                                    explorer={"https://soneium-minato.blockscout.com/address/"}/>
          <LegendParticipationWorkflow cx={700}/>
        </svg>
        <CloseParticipations rpc={rpcShibuya} address={managerAddress}/>
      </header>
    </div>
  );
}
