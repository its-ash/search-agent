import type { Citation } from "../types/contracts";

export function SourcesPanel({ citations }: { citations: Citation[] }) {
  if (!citations.length) return <p>No sources.</p>;

  return (
    <ul>
      {citations.map((c) => (
        <li key={c.chunkId}>
          <strong>{basename(c.file)}</strong>
        </li>
      ))}
    </ul>
  );
}

function basename(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}
