import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcAstar = "wss://rpc.astar.network";
const managerAddress = "Y4j9j5EsehgPAjL6HHMMjHU9EhCdwrADazXEQdhFciG4uR9";
const registrationAstar ="ZsaaL58Adg7k1xT1EumK575H6GymJ214kvRk8NxKouTWqnC";
const rpcSoneium = "https://rpc.soneium.org";
const registrationSoneium = "0xB2196C9B95BD3cdC799eb89f856895aEDbd649bB";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0xa41292a45bb4103c72796409d557416cc259ffb420a94fb433ae4c942fdc1218";

export default function App() {
  return (
    <div className="App">
      <header className="App-header">
        <svg width="1000" height="1000">
          <LegendManagerWorkflow cx={0}/>
          <ManagerWorkflow cx={250} rpc={rpcAstar} address={managerAddress} chain={"Astar"}
                           explorer={"https://astar.subscan.io/wasm_contract/"}
          />
          <ParticipationWorkflowWasm cx={500} rpc={rpcAstar} address={registrationAstar} chain={"Astar"}
                                     explorer={"https://astar.subscan.io/wasm_contract/"}/>
          <ParticipationWorkflowEvm cx={600} rpc={rpcSoneium} address={registrationSoneium} chain={"Soneium"}
                                    explorer={"https://soneium.blockscout.com/address/"}/>
          <LegendParticipationWorkflow cx={700}/>
          <Synchronisation rpcManagerContract={rpcAstar} addressManagerContract={managerAddress}
                           rpcCommunicatingContract={rpcPhala} addressCommunicatingContract={pinkContractAddress}
                           explorerCommunicatingContract={"https://phala.subscan.io/wasm_contract/"}/>
        </svg>
      </header>
    </div>
  );
}
