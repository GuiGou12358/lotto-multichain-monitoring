import {useEffect, useState} from "react";
import {RaffleManagerWasm} from "./wasmContract";
import {LottoDraw} from "./lottoDraw";

export function Synchronisation( {rpcManagerContract, addressManagerContract, rpcCommunicatingContract, addressCommunicatingContract, explorerCommunicatingContract }
) {

  const [isEnabledSynchronization, enableSynchronization] = useState(false);
  const [inProgress, setInProgress] = useState(false);
  const [nextClosingRegistrations, setNextClosingRegistrations] = useState(0);
  const [canCloseRegistrations, setCanCloseRegistrations] = useState(false);
  const [currentBlock, setCurrentBlock] = useState(0);

  const raffleManager = new RaffleManagerWasm(rpcManagerContract, addressManagerContract);
  const synchronizer = new LottoDraw(rpcCommunicatingContract, addressCommunicatingContract);

  const synchronizeInBackground = async () => {
    try {

      await raffleManager.init();
      await synchronizer.init();

      setCurrentBlock(await raffleManager.getCurrentBlock());
      setNextClosingRegistrations(await raffleManager.getNextClosingRegistrations());
      const canCloseRegistrations = await raffleManager.canCloseRegistrations();
      setCanCloseRegistrations(canCloseRegistrations);

      if (!isEnabledSynchronization){
        setInProgress(false);
        return;
      }

      const hasPendingMessage = await raffleManager.hasPendingMessage();
      setInProgress(hasPendingMessage || canCloseRegistrations);

      if (hasPendingMessage) {
        await synchronizer.synchronizeContracts();
      }

      if (canCloseRegistrations) {
        await synchronizer.closeRegistrations();
      }

    } catch (e){
      console.error(e);
    }
  };

  useEffect(() => {
    const backgroundSyncInterval = setInterval(() => {
      synchronizeInBackground();
    }, 60 * 1000); // every 60 seconds

    return () => {
      clearInterval(backgroundSyncInterval);
    }
  });

  const enableSynchronisation = () => {
    enableSynchronization(!isEnabledSynchronization);
  };

  return (
    <>
      <defs>
        <linearGradient id="animatedGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="black">
            <animate attributeName="stop-color" values="black;gray;black" dur="2s" repeatCount="indefinite"/>
          </stop>
          <stop offset="100%" stop-color="gray">
            <animate attributeName="stop-color" values="gray;black;gray" dur="2s" repeatCount="indefinite"/>
          </stop>
        </linearGradient>
      </defs>
      <rect x="180" y="850" width="500" height="50"
            fill={isEnabledSynchronization && inProgress ? "url(#animatedGradient)" : "gray"}/>
      <text x="430" y="875" fill="white" fontSize="14" textAnchor="middle" dominantBaseline="middle">
        {!isEnabledSynchronization ? "Synchronisation disabled"
          : inProgress ? "Synchronisation in progress - communication with smart contracts"
            : "Waiting synchronisation"}
      </text>
      <a href={explorerCommunicatingContract + addressCommunicatingContract} target="_blank" rel="noreferrer noopener">
        <text x="830" y="890" fill="black" fontSize="14" textAnchor="middle" dominantBaseline="middle">
          Via a contract on Phala testnet
        </text>
      </a>
      <rect x="720" y="860" width="20" height="20" fill="white" stroke={"black"}
            onClick={enableSynchronisation}/>
      <path d="M722 865 L732 875 L740 860" stroke={isEnabledSynchronization ? "black" : "none"} strokeWidth="3"
            fill="none"/>
      <text x="750" y="875" fill="white" fontSize="14">
        Enable synchronisation
      </text>
      <text x="200" y="925" fill="white" fontSize="14">
        {canCloseRegistrations ?
          "The draw can start ! Please enable synchronisation to start it."
          : "Closing the registration in " +
          ((nextClosingRegistrations > currentBlock) ? nextClosingRegistrations - currentBlock : '-')
          + " blocks (ie at block: " + nextClosingRegistrations + ", current block: " + currentBlock + ")"
        }
      </text>
    </>
  );
}
