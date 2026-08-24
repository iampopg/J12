import { useState, useEffect } from "react";

export interface ScanState {
  scanning: boolean;
  progress: number;
  stage: string;
}

let state: ScanState = {
  scanning: false,
  progress: 0,
  stage: "",
};

const listeners = new Set<() => void>();

export const scanStore = {
  getState: (): ScanState => state,
  setState: (updates: Partial<ScanState>) => {
    state = { ...state, ...updates };
    listeners.forEach((fn) => fn());
  },
  subscribe: (listener: () => void) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
};

export function useScanState() {
  const [scanState, setLocalState] = useState<ScanState>(() => scanStore.getState());

  useEffect(() => {
    return scanStore.subscribe(() => {
      setLocalState({ ...scanStore.getState() });
    });
  }, []);

  const updateScanState = (updates: Partial<ScanState>) => {
    scanStore.setState(updates);
  };

  return [scanState, updateScanState] as const;
}
