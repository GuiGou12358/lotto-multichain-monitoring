import {RaffleManagerWasm} from "./wasmContract";
import {useEffect, useState} from "react";
import {LottoDraw} from "./lottoDraw";

export function SynchronizeButton({rpcManager, addressManager, rpcPinkContract, addressPinkContract}) {

  const [enabledButton, setEnabledButton] = useState(false);

  useEffect(() => {
    const syncStatusInBackground = async () => {
      try {
        const contract = new RaffleManagerWasm(rpcManager, addressManager);
        await contract.init();
        setEnabledButton(await contract.hasPendingMessage());
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
  }, [rpcManager, addressManager]);

  const synchronizeContracts = async () => {
    try {
      const contract = new LottoDraw(rpcPinkContract, addressPinkContract);
      await contract.init();
      await contract.synchronizeContracts();
    } catch (e){
      console.error(e);
    }
  };

  return (
    <button onClick={synchronizeContracts} disabled={!enabledButton}>Synchronize</button>
  );
}

