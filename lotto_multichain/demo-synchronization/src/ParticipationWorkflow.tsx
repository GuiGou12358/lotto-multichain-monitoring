import {cy, fillSelected, fillUnselected, fontSize, legendColor, r} from "./constants";
import {RaffleRegistrationWasm} from "./wasmContract";
import {useEffect, useState} from "react";
import {ParticipationEvm} from "./evmContract";

let wasmStatuses = new Map<string, number>([
  ["NotStarted", 0],
  ["Started", 1],
  ["RegistrationsOpen", 2],
  ["RegistrationsClosed", 3],
  ["ResultsReceived", 4],
]);

function getStatusColor(current : string, expected : number){
  if (current === expected.toString() || wasmStatuses.get(current) === expected){
    return fillSelected;
  }
  return fillUnselected
}


function ParticipationWorkflow({cx, status, drawNumber, chain, explorer, address}) {
  return (
    <>
      <a href={explorer+address} target="_blank" rel="noreferrer noopener">
        <text x={cx} y={cy} className="contract">
          <tspan x={cx - 30} dy={15}>Participation</tspan>
          <tspan x={cx - 20} dy={20} fill={"black"}>{chain}</tspan>
        </text>
      </a>

      <circle cx={cx} cy={2 * cy} r={r} fill={getStatusColor(status, 0)}></circle>
      <line x1={cx} y1={2 * cy + r} x2={cx} y2={3 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>

      <circle cx={cx} cy={3 * cy} r={r} fill={getStatusColor(status, 1)}></circle>
      <line x1={cx} y1={3 * cy + r} x2={cx} y2={4 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>

      <circle cx={cx} cy={4 * cy} r={r} fill={getStatusColor(status, 2)}></circle>
      <line x1={cx} y1={4 * cy + r} x2={cx} y2={5 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>
      <circle cx={cx} cy={5 * cy} r={r} fill={getStatusColor(status, 3)}></circle>
      <line x1={cx} y1={5 * cy + r} x2={cx} y2={7 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>
      <circle cx={cx} cy={7 * cy} r={r} fill={getStatusColor(status, 4)}></circle>

      <text x={cx} y={8 * cy} fontSize={fontSize} fill={legendColor}>{drawNumber}</text>

    </>
  );

}

export function LegendParticipationWorkflow({cx}: { cx: number }) {
  return (
    <>
      <text x={cx} y={2 * cy + 7} fontSize={fontSize} fill={legendColor}>Instantiated</text>
      <text x={cx} y={3 * cy + 7} fontSize={fontSize} fill={legendColor}>Configured</text>
      <text x={cx} y={4 * cy + 7} fontSize={fontSize} fill={legendColor}>Registrations open (the user can participate)</text>
      <text x={cx} y={5 * cy + 7} fontSize={fontSize} fill={legendColor}>Registrations closed</text>
      <text x={cx} y={7 * cy + 7} fontSize={fontSize} fill={legendColor}>Results known</text>
    </>
  );
}

export function ParticipationWorkflowWasm({ cx, rpc, address, chain, explorer }) {

  const[status, setStatus] = useState("");
  const[drawNumber, setDrawNumber] = useState("");

  const contract = new RaffleRegistrationWasm(rpc, address);
  const syncDataInBackground = async () => {
    try {
      await contract.init();
      setStatus(await contract.getStatus());
      setDrawNumber(await contract.getDrawNumber());
    } catch (e){
      console.error(e);
    }
  };

  useEffect( () => {
    const backgroundSyncInterval = setInterval(() => {
      syncDataInBackground();
    }, 15 * 1000); // every 15 seconds

    return () => {
      clearInterval(backgroundSyncInterval);
    }
  });

  return ParticipationWorkflow({cx, status, drawNumber, chain, address, explorer});
}

export function ParticipationWorkflowEvm({cx, rpc, address, chain, explorer}) {

  const[status, setStatus] = useState("");
  const[drawNumber, setDrawNumber] = useState("");

  const contract = new ParticipationEvm(rpc, address);
  const syncDataInBackground = async () => {
    try {
      await contract.init();
      setStatus(await contract.getStatus());
      setDrawNumber(await contract.getDrawNumber());
    } catch (e){
      console.error(e);
    }
  };

  useEffect( () => {
    const backgroundSyncInterval = setInterval(() => {
      syncDataInBackground();
    }, 15 * 1000); // every 15 seconds

    return () => {
      clearInterval(backgroundSyncInterval);
    }
  });

  return ParticipationWorkflow({cx, status, drawNumber, chain, address, explorer});
}
