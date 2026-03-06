import { invoke } from "@tauri-apps/api/core";
import type { AskResponse, IndexStats, ScanStatus, ServerStatusResponse } from "../types/contracts";

export async function startScan(rootPath: string): Promise<{ jobId: string }> {
  return invoke("start_scan", { request: { rootPath } });
}

export async function cancelScan(jobId: string): Promise<{ ok: boolean }> {
  return invoke("cancel_scan", { request: { jobId } });
}

export async function getScanStatus(jobId: string): Promise<ScanStatus> {
  return invoke("get_scan_status", { request: { jobId } });
}

export async function askQuestion(question: string): Promise<AskResponse> {
  return invoke("ask_question", { request: { question } });
}

export async function getServerStatus(): Promise<ServerStatusResponse> {
  return invoke("get_server_status");
}

export async function restartServer(): Promise<{ ok: boolean }> {
  return invoke("restart_server");
}

export async function rebuildIndex(): Promise<{ jobId: string }> {
  return invoke("rebuild_index", { request: {} });
}

export async function resetIndex(): Promise<{ ok: boolean }> {
  return invoke("reset_index");
}

export async function getIndexStats(): Promise<IndexStats> {
  return invoke("get_index_stats");
}
