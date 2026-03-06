import { create } from "zustand";
import type { ServerStatusResponse } from "../types/contracts";
import { getServerStatus, restartServer } from "../api/tauri";

type ServerStore = {
  status: ServerStatusResponse;
  refresh: () => Promise<void>;
  restart: () => Promise<void>;
};

export const useServerStore = create<ServerStore>((set) => ({
  status: { status: "stopped", ownership: "none", endpoint: "http://127.0.0.1:8080" },
  refresh: async () => {
    const status = await getServerStatus();
    set({ status });
  },
  restart: async () => {
    await restartServer();
    const status = await getServerStatus();
    set({ status });
  }
}));
