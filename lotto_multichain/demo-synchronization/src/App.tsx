import './App.css';
import {CloseParticipation, LegendManagerWorkflow, ManagerWorkflow} from "./ManagerWorkflow";
import {
  LegendParticipationWorkflow,
  ParticipationWorkflowEvm,
  ParticipationWorkflowWasm
} from "./ParticipationWorkflow";
import {Synchronisation} from "./Synchronisation";

const rpcShibuya = "wss://rpc.shibuya.astar.network";
const managerAddress = "aSAnXcJ2QnrncaMsW4dZXY2vC6XTbvjjBaUhznFNLQoByzL";
const registration1Address ="XYsnkvzKqXcVJgAUYpK8MfWaFiTuCEzbP42L5NGVGXHfNbC";
const rpcMoonbase = "https://rpc.api.moonbase.moonbeam.network";
const registration2Address = "0x879A1dd3f4f968dD0b2D54e4d5F08AE41cC318c3";
const rpcMinato = "https://rpc.minato.soneium.org";
const registration3Address = "0x100389C2bA0A9F22B2bEAa8bC976Ca00e63B3724";
const rpcPhala = "wss://poc6.phala.network/ws";
const pinkContractAddress = "0x2ccd40a7cda610492b1634687a2953ac31c0b7f706d2f55f9ffee9793a7d8e74";

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
        <CloseParticipation rpc={rpcShibuya} address={managerAddress}/>
      </header>
    </div>
  );
}
