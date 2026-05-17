import type { ProxyMode } from "./proxy";

export interface AppConfig {
  mixed_port: number;
  controller_port: number;
  secret: string;
  mode: ProxyMode;
  auto_start: boolean;
}

export interface SystemProxyStatus {
  enabled: boolean;
  http_proxy: string | null;
  https_proxy: string | null;
  socks_proxy: string | null;
}
