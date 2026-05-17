export type ProxyMode = "global" | "rule" | "direct";
export type CoreStatus = "stopped" | "starting" | "running" | "error";

export interface ProxyNode {
  name: string;
  type: string;
  udp: boolean;
  history: { time: string; delay: number }[];
}

export interface ProxyGroup {
  name: string;
  type: string;
  now: string;
  all: string[];
}

export interface MihomoStatus {
  running: boolean;
  version: string | null;
}
