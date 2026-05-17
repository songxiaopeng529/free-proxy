import { create } from "zustand";
import type { ProxyMode, ProxyGroup, CoreStatus } from "../types/proxy";
import { commands } from "../services/tauriCommands";
import { mihomoApi, setSecret } from "../services/mihomoApi";

interface ProxyState {
  enabled: boolean;
  mode: ProxyMode;
  groups: ProxyGroup[];
  coreStatus: CoreStatus;
  loading: boolean;

  toggleProxy: () => Promise<void>;
  setMode: (mode: ProxyMode) => Promise<void>;
  selectNode: (group: string, name: string) => Promise<void>;
  fetchNodes: () => Promise<void>;
  fetchStatus: () => Promise<void>;
  testGroupDelay: (group: string) => Promise<Record<string, number>>;
}

export const useProxyStore = create<ProxyState>((set, get) => ({
  enabled: false,
  mode: "rule",
  groups: [],
  coreStatus: "stopped",
  loading: false,

  toggleProxy: async () => {
    const { enabled } = get();
    set({ loading: true });
    try {
      if (enabled) {
        try {
          await commands.disableSystemProxy();
        } catch (e) {
          console.warn("disableSystemProxy failed:", e);
        }
        await commands.stopProxy();
        set({ enabled: false, coreStatus: "stopped", groups: [] });
      } else {
        set({ coreStatus: "starting" });
        await commands.startProxy();
        const config = await commands.getAppConfig();
        setSecret(config.secret);
        try {
          await commands.enableSystemProxy();
        } catch (e) {
          console.warn("enableSystemProxy failed:", e);
        }
        set({ enabled: true, coreStatus: "running", mode: config.mode });
        await get().fetchNodes();
      }
    } catch (e) {
      console.error("toggleProxy error:", e);
      set({ coreStatus: "error" });
    } finally {
      set({ loading: false });
    }
  },

  setMode: async (mode) => {
    try {
      await commands.setProxyMode(mode);
      set({ mode });
    } catch (e) {
      console.error("setMode error:", e);
    }
  },

  selectNode: async (group, name) => {
    try {
      await mihomoApi.selectProxy(group, name);
      set((state) => ({
        groups: state.groups.map((g) =>
          g.name === group ? { ...g, now: name } : g,
        ),
      }));
    } catch (e) {
      console.error("selectNode error:", e);
    }
  },

  fetchNodes: async () => {
    try {
      const data = await mihomoApi.getProxies();
      const proxies = data.proxies || {};
      const groups: ProxyGroup[] = [];

      for (const [name, proxy] of Object.entries(proxies) as [
        string,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        any,
      ][]) {
        if (proxy.all && proxy.all.length > 0) {
          groups.push({
            name,
            type: proxy.type,
            now: proxy.now || "",
            all: proxy.all,
          });
        }
      }

      set({ groups });
    } catch (e) {
      console.error("fetchNodes error:", e);
    }
  },

  fetchStatus: async () => {
    try {
      const status = await commands.getProxyStatus();
      if (status.running) {
        const config = await commands.getAppConfig();
        setSecret(config.secret);
        const sysProxy = await commands.getSystemProxyStatus();
        set({
          coreStatus: "running",
          enabled: sysProxy.enabled,
          mode: config.mode,
        });
        await get().fetchNodes();
      } else {
        set({ coreStatus: "stopped", enabled: false });
      }
    } catch {
      set({ coreStatus: "stopped", enabled: false });
    }
  },

  testGroupDelay: async (group) => {
    try {
      const delays: Record<string, number> = await mihomoApi.testGroupDelay(group);
      return delays;
    } catch (e) {
      console.error("testGroupDelay error:", e);
      return {};
    }
  },
}));
