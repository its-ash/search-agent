export type ServerStatus = "starting" | "running" | "stopped" | "error";
export type Ownership = "self" | "external" | "none";

export type ServerStatusResponse = {
  status: ServerStatus;
  ownership: Ownership;
  endpoint: string;
  pid?: number;
  message?: string;
};

export type ScanState = "idle" | "scanning" | "indexing" | "ready" | "error" | "canceled";

export type ScanStatus = {
  jobId: string;
  status: ScanState;
  totalFiles: number;
  processedFiles: number;
  failedFiles: number;
  currentFile?: string;
  failures: FailureEntry[];
};

export type FailureEntry = {
  path: string;
  stage: string;
  message: string;
  retryable: boolean;
};

export type Citation = {
  file: string;
  pageStart?: number;
  pageEnd?: number;
  chunkId: string;
  excerpt: string;
};

export type AskResponse = {
  answer: string;
  notFound: boolean;
  citations: Citation[];
  latencyMs: number;
};

export type IndexStats = {
  indexedFiles: number;
  indexedChunks: number;
};
