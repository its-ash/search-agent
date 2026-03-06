import { useScanStore } from "../store/scanStore";

export function ErrorPanel() {
  const failures = useScanStore((s) => s.status.failures);

  return (
    <div className="panel">
      <h3>Failures</h3>
      {failures.length === 0 ? <p>No failures.</p> : (
        <ul>
          {failures.map((f, i) => (
            <li key={`${f.path}-${i}`}>{f.path} [{f.stage}] {f.message}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
