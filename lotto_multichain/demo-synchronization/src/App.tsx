import './App.css';
import {LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "bLQBJHeqGUPS1zJgGeEG4yR9pGfom4Sr5j1QSnoGveH86Rz";
const registration1Address ="ZEDzKmBFeCPpjyfTt7aHVVBfM8Cu8i8psj2Z7TEsgnXTBLq";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0x100389C2bA0A9F22B2bEAa8bC976Ca00e63B3724";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0xcB40e3F70c591A230Ce2E263C07682fDd8a784E9";
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
                           rpcCommunicatingContract={rpcPhala} addressCommunicatingContract={pinkContractAddress}
                           explorerCommunicatingContract={"https://phala.subscan.io/wasm_contract/"}/>
        </svg>
      </header>
    </div>
  );
}
