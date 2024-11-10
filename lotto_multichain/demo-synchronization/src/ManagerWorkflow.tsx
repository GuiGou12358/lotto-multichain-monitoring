import {cy, fillSelected, fillUnselected, fontSize, legendColor, r} from "./constants";
import {RaffleManagerWasm} from "./wasmContract";
import {useEffect, useState} from "react";

function getStatusColor(current : string, expected : string){
    return current === expected ? fillSelected : fillUnselected;
}


export function LegendManagerWorkflow({ cx }) {
  return (
    <>
      <text x={cx} y={2 * cy + 7} fontSize={fontSize} fill={legendColor}>Instantiated</text>
      <text x={cx} y={3 * cy + 7} fontSize={fontSize} fill={legendColor}>Configured</text>
      <text x={cx} y={4 * cy + 7} fontSize={fontSize} fill={legendColor}>Registrations open</text>
      <text x={cx} y={5 * cy + 7} fontSize={fontSize} fill={legendColor}>Registrations closed</text>
      <text x={cx} y={6 * cy + 7} fontSize={fontSize} fill={legendColor}>Lotto draw done - checking winners</text>
      <text x={cx} y={7 * cy + 7} fontSize={fontSize} fill={legendColor}>Results received</text>
      <text x={cx} y={8 * cy} fontSize={fontSize} fill={legendColor}>Draw Number</text>
    </>
  );
}


export function ManagerWorkflow({cx, rpc, address, chain}) {

  const [status, setStatus] = useState("0");
  const [drawNumber, setDrawNumber] = useState("");

  useEffect(() => {
    const syncStatusInBackground = async () => {
      try {
        const contract = new RaffleManagerWasm(rpc, address);
        await contract.init();
        setStatus(await contract.getStatus());
        setDrawNumber(await contract.getDrawNumber());
      } catch (e){
        console.error(e);
      }
    };

    const backgroundSyncInterval = setInterval(() => {
      syncStatusInBackground();
    }, 15 * 1000); // every 15 seconds

    return () => {
      clearInterval(backgroundSyncInterval);
    }
  }, []);

  return (
    <>
      <text x={cx} y={cy} className="contract">
        <tspan x={cx - 30} dy={15}>Lotto Manager</tspan>
        <tspan x={cx - 20} dy={20} fill={"black"}>{chain}</tspan>
      </text>

      <circle cx={cx} cy={2 * cy} r={r} fill={getStatusColor(status, "NotStarted")}></circle>
      <line x1={cx} y1={2 * cy + r} x2={cx} y2={3 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>

      <circle cx={cx} cy={3 * cy} r={r} fill={getStatusColor(status, "Started")}></circle>
      <line x1={cx} y1={3 * cy + r} x2={cx} y2={4 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>

      <circle cx={cx} cy={4 * cy} r={r} fill={getStatusColor(status, "RegistrationsOpen")}></circle>
      <line x1={cx} y1={4 * cy + r} x2={cx} y2={5 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>
      <circle cx={cx} cy={5 * cy} r={r} fill={getStatusColor(status, "RegistrationsClosed")}></circle>
      <line x1={cx} y1={5 * cy + r} x2={cx} y2={6 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>
      <circle cx={cx} cy={6 * cy} r={r} fill={getStatusColor(status, "WaitingWinners")}></circle>
      <line x1={cx} y1={6 * cy + r} x2={cx} y2={7 * cy - r - 5} stroke="black" marker-end="url(#arrowhead)"></line>
      <defs>
        <marker id="arrowhead" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="black"></path>
        </marker>
      </defs>
      <circle cx={cx} cy={7 * cy} r={r} fill={getStatusColor(status, "Closed")}></circle>

      <text x={cx} y={8 * cy} fontSize={fontSize} fill={legendColor}>{drawNumber}</text>
    </>
  );
}
