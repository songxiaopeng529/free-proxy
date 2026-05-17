import { create } from "zustand";
import type { Subscription } from "../types/subscription";
import { commands } from "../services/tauriCommands";

interface SubscriptionState {
  subscriptions: Subscription[];
  loading: boolean;

  fetchSubscriptions: () => Promise<void>;
  addSubscription: (url: string, name?: string) => Promise<void>;
  removeSubscription: (id: string) => Promise<void>;
  updateSubscription: (id: string) => Promise<void>;
  updateAll: () => Promise<void>;
}

export const useSubscriptionStore = create<SubscriptionState>((set) => ({
  subscriptions: [],
  loading: false,

  fetchSubscriptions: async () => {
    try {
      const subs = await commands.listSubscriptions();
      set({ subscriptions: subs });
    } catch {
      // not running yet
    }
  },

  addSubscription: async (url, name) => {
    set({ loading: true });
    try {
      const sub = await commands.addSubscription(url, name);
      set((state) => ({ subscriptions: [...state.subscriptions, sub] }));
    } finally {
      set({ loading: false });
    }
  },

  removeSubscription: async (id) => {
    await commands.removeSubscription(id);
    set((state) => ({
      subscriptions: state.subscriptions.filter((s) => s.id !== id),
    }));
  },

  updateSubscription: async (id) => {
    set({ loading: true });
    try {
      const updated = await commands.updateSubscription(id);
      set((state) => ({
        subscriptions: state.subscriptions.map((s) =>
          s.id === id ? updated : s,
        ),
      }));
    } finally {
      set({ loading: false });
    }
  },

  updateAll: async () => {
    set({ loading: true });
    try {
      const subs = await commands.updateAllSubscriptions();
      set({ subscriptions: subs });
    } finally {
      set({ loading: false });
    }
  },
}));
