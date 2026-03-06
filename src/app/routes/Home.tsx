import { FolderPicker } from "../components/FolderPicker";
import { ScanControls } from "../components/ScanControls";
import { ScanProgress } from "../components/ScanProgress";
import { ErrorPanel } from "../components/ErrorPanel";
import { ServerStatusBadge } from "../components/ServerStatusBadge";
import { ChatPanel } from "../components/ChatPanel";

export function Home() {
  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <h1>SearchAgent</h1>
          <p>Local-first RAG Desktop</p>
        </div>
        <ServerStatusBadge />
        <FolderPicker />
        <ScanControls />
        <ScanProgress />
        <ErrorPanel />
      </aside>
      <main className="chat-main">
        <ChatPanel />
      </main>
    </div>
  );
}
