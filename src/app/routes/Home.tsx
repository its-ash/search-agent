import { FolderPicker } from "../components/FolderPicker";
import { ScanControls } from "../components/ScanControls";
import { ScanProgress } from "../components/ScanProgress";
import { ErrorPanel } from "../components/ErrorPanel";
import { ServerStatusBadge } from "../components/ServerStatusBadge";
import { ChatPanel } from "../components/ChatPanel";
import logo from "../../assets/search-agent-logo.svg";

export function Home() {
  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-row">
            <img src={logo} alt="Search Agent logo" className="brand-logo" />
            <h1>Search Agent</h1>
          </div>
        </div>
        <ServerStatusBadge />
        <FolderPicker />
        <ScanControls />
        <ScanProgress />
        <ErrorPanel />
      </aside>
      <main className="chat-main">
        <div className="chat-shell">
          <ChatPanel />
        </div>
      </main>
    </div>
  );
}
