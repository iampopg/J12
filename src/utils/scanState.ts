import { useState, useEffect } from "react";

export interface ScanState {
  scanning: boolean;
  progress: number;
  stage: string;
  startedAt?: number;
}

let state: ScanState = {
  scanning: false,
  progress: 0,
  stage: "",
};

let autoResetTimer: any = null;

const listeners = new Set<() => void>();

export const scanStore = {
  getState: (): ScanState => state,
  setState: (updates: Partial<ScanState>) => {
    state = { 
      ...state, 
      ...updates,
      startedAt: updates.scanning ? (state.startedAt || Date.now()) : undefined,
    };
    
    // Auto-expire scanning after 20 seconds to prevent UI lockup
    if (autoResetTimer) clearTimeout(autoResetTimer);
    if (state.scanning) {
      autoResetTimer = setTimeout(() => {
        scanStore.reset();
      }, 20000);
    }

    listeners.forEach((fn) => fn());
  },
  reset: () => {
    if (autoResetTimer) clearTimeout(autoResetTimer);
    state = {
      scanning: false,
      progress: 0,
      stage: "",
    };
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

  return [scanState, updateScanState, scanStore.reset] as const;
}
