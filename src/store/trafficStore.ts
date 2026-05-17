import { create } from "zustand";

interface TrafficState {
  upload: number;
  download: number;
  memory: number;
  setTraffic: (up: number, down: number) => void;
  setMemory: (mem: number) => void;
}

export const useTrafficStore = create<TrafficState>((set) => ({
  upload: 0,
  download: 0,
  memory: 0,
  setTraffic: (upload, download) => set({ upload, download }),
  setMemory: (memory) => set({ memory }),
}));
