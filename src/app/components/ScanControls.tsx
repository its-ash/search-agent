import { rebuildIndex, resetIndex } from "../api/tauri";
import { useScanStore } from "../store/scanStore";

export function ScanControls() {
  const launchScan = useScanStore((s) => s.launchScan);
  const cancel = useScanStore((s) => s.cancel);

  return (
    <div className="panel">
      <h3>Scan Controls</h3>
      <div className="row">
        <button className="btn btn-primary" onClick={() => void launchScan()}>Scan</button>
        <button className="btn btn-secondary" onClick={() => void cancel()}>Cancel</button>
        <button className="btn btn-secondary" onClick={() => void rebuildIndex()}>Rebuild Index</button>
        <button className="btn btn-secondary" onClick={() => void resetIndex()}>Reset Index</button>
      </div>
    </div>
  );
}
