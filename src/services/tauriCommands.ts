import { invoke } from "@tauri-apps/api/core";
import type { Subscription } from "../types/subscription";
import type { MihomoStatus, ProxyGroup } from "../types/proxy";
import type { AppConfig, SystemProxyStatus } from "../types/config";

export const commands = {
  startProxy: () => invoke<void>("start_proxy"),
  stopProxy: () => invoke<void>("stop_proxy"),
  restartProxy: () => invoke<void>("restart_proxy"),
  getProxyStatus: () => invoke<MihomoStatus>("get_proxy_status"),

  addSubscription: (url: string, name?: string) =>
    invoke<Subscription>("add_subscription", { url, name }),
  removeSubscription: (id: string) =>
    invoke<void>("remove_subscription", { id }),
  updateSubscription: (id: string) =>
    invoke<Subscription>("update_subscription", { id }),
  updateAllSubscriptions: () =>
    invoke<Subscription[]>("update_all_subscriptions"),
  listSubscriptions: () => invoke<Subscription[]>("list_subscriptions"),

  getAppConfig: () => invoke<AppConfig>("get_app_config"),
  setProxyMode: (mode: string) => invoke<void>("set_proxy_mode", { mode }),
  getProxyGroups: () => invoke<ProxyGroup[]>("get_proxy_groups"),
  selectProxy: (group: string, name: string) =>
    invoke<void>("select_proxy", { group, name }),

  enableSystemProxy: () => invoke<void>("enable_system_proxy"),
  disableSystemProxy: () => invoke<void>("disable_system_proxy"),
  getSystemProxyStatus: () =>
    invoke<SystemProxyStatus>("get_system_proxy_status"),

  getRules: () => invoke<string[]>("get_rules"),
  setCustomRules: (rules: string[]) =>
    invoke<void>("set_custom_rules", { rules }),
};
