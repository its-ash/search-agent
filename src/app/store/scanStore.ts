import { create } from "zustand";
import type { ScanStatus } from "../types/contracts";
import { cancelScan, getScanStatus, startScan } from "../api/tauri";

type ScanStore = {
  rootPath: string;
  jobId?: string;
  status: ScanStatus;
  setRootPath: (path: string) => void;
  launchScan: () => Promise<void>;
  refresh: () => Promise<void>;
  cancel: () => Promise<void>;
};

const emptyStatus: ScanStatus = {
  jobId: "",
  status: "idle",
  totalFiles: 0,
  processedFiles: 0,
  failedFiles: 0,
  failures: []
};

export const useScanStore = create<ScanStore>((set, get) => ({
  rootPath: "",
  jobId: undefined,
  status: emptyStatus,
  setRootPath: (path) => set({ rootPath: path }),
  launchScan: async () => {
    const { rootPath } = get();
    if (!rootPath) return;
    const { jobId } = await startScan(rootPath);
    set({ jobId });
    await get().refresh();
  },
  refresh: async () => {
    const { jobId } = get();
    if (!jobId) return;
    const status = await getScanStatus(jobId);
    set({ status });
  },
  cancel: async () => {
    const { jobId } = get();
    if (!jobId) return;
    await cancelScan(jobId);
    await get().refresh();
  }
}));
