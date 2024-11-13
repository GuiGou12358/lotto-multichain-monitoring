import {useEffect, useState} from "react";
import {RaffleManagerWasm} from "./wasmContract";
import {LottoDraw} from "./lottoDraw";

export function Synchronisation( {rpcManagerContract, addressManagerContract, rpcCommunicatingContract, addressCommunicatingContract }
) {

  const [isEnabledSynchronization, enableSynchronization] = useState(false);
  const [inProgress, setInProgress] = useState(false);
  const [nextClosingRegistrations, setNextClosingRegistrations] = useState(0);
  const [currentBlock, setCurrentBlock] = useState(0);

  const raffleManager = new RaffleManagerWasm(rpcManagerContract, addressManagerContract);
  const synchronizer = new LottoDraw(rpcCommunicatingContract, addressCommunicatingContract);

  const synchronizeInBackground = async () => {
    try {

      await raffleManager.init();
      await synchronizer.init();

      setCurrentBlock(await raffleManager.getCurrentBlock());
      setNextClosingRegistrations(await raffleManager.getNextClosingRegistrations());

      if (!isEnabledSynchronization){
        setInProgress(false);
        return;
      }

      const hasPendingMessage = await raffleManager.hasPendingMessage();
      const canCloseRegistrations = await raffleManager.canCloseRegistrations();
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
    }, 15 * 1000); // every 15 seconds

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
      <rect x="180" y="700" width="500" height="50"
            fill={isEnabledSynchronization && inProgress ? "url(#animatedGradient)" : "gray"}/>
      <text x="430" y="725" fill="white" fontSize="14" textAnchor="middle" dominantBaseline="middle">
        {!isEnabledSynchronization ? "Synchronisation disabled"
          : inProgress ? "Synchronisation in progress - communication with smart contracts"
            : "Waiting synchronisation"}
      </text>

      <rect x="720" y="710" width="20" height="20" fill="white" stroke={"black"}
            onClick={enableSynchronisation}/>
      <path d="M722 715 L732 725 L740 710" stroke={isEnabledSynchronization ? "black" : "none"} strokeWidth="3"
            fill="none"/>
      <text x="750" y="725" fill="white" fontSize="14">
        Enable synchronisation
      </text>

      <text x="180" y="775" fill="white" fontSize="14">
        next closing registration {nextClosingRegistrations} - current Block : {currentBlock}
      </text>
    </>
  );
}
