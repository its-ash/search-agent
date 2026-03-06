import { useEffect, useState } from "react";
import { getIndexStats } from "../api/tauri";
import type { IndexStats } from "../types/contracts";
import { useScanStore } from "../store/scanStore";

export function ScanProgress() {
  const status = useScanStore((s) => s.status);
  const refresh = useScanStore((s) => s.refresh);
  const [stats, setStats] = useState<IndexStats>({ indexedFiles: 0, indexedChunks: 0 });

  useEffect(() => {
    const sync = async () => {
      await refresh();
      const res = await getIndexStats();
      setStats(res);
    };

    void sync();
    const id = setInterval(() => void sync(), 1000);
    return () => clearInterval(id);
  }, [refresh]);

  const pct = status.totalFiles > 0 ? Math.round((status.processedFiles / status.totalFiles) * 100) : 0;

  return (
    <div className="panel">
      <h3>Scan Status: {status.status}</h3>
      <div className="progress"><span style={{ width: `${pct}%` }} /></div>
      <p>{status.processedFiles}/{status.totalFiles} processed, {status.failedFiles} failed</p>
      <p className="meta-line">Indexed files: {stats.indexedFiles} | Chunks: {stats.indexedChunks}</p>
      {status.currentFile ? <p>Current: {status.currentFile}</p> : null}
    </div>
  );
}
