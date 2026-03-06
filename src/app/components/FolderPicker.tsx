import { open } from "@tauri-apps/plugin-dialog";
import { useScanStore } from "../store/scanStore";

export function FolderPicker() {
  const rootPath = useScanStore((s) => s.rootPath);
  const setRootPath = useScanStore((s) => s.setRootPath);

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") setRootPath(selected);
  }

  return (
    <div className="panel">
      <h3>Folder</h3>
      <div className="row">
        <input style={{ flex: 1 }} value={rootPath} readOnly placeholder="Select a folder" />
        <button className="btn btn-secondary" onClick={pickFolder}>Browse</button>
      </div>
    </div>
  );
}
