import { useEffect } from "react";
import { useServerStore } from "../store/serverStore";

export function ServerStatusBadge() {
  const status = useServerStore((s) => s.status);
  const refresh = useServerStore((s) => s.refresh);
  const restart = useServerStore((s) => s.restart);

  useEffect(() => {
    void refresh();
    const id = setInterval(() => void refresh(), 2000);
    return () => clearInterval(id);
  }, [refresh]);

  return (
    <div className="panel row" style={{ justifyContent: "space-between" }}>
      <div>
        <strong>Model Server</strong>
        <span
          className={`badge ${status.status} camel-case`}
          style={{ marginLeft: 8 }}
        >
          <span className="camel-case">{status.status}</span> {status.ownership}
        </span>
      </div>
      <button className="btn btn-secondary" onClick={() => void restart()}>
        Restart
      </button>
    </div>
  );
}
