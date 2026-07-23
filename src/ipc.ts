import { invoke, type Channel } from "@tauri-apps/api/core";

export type AppError = {
  code: string;
  message: string;
  retryable: boolean;
  action?: string;
};

export type AppSettings = {
  activeModelConfigId: string | null;
  activePromptId: string | null;
  captureShortcut: string;
  saveHistory: boolean;
  autostart: boolean;
  resultAlwaysOnTop: boolean;
  onboardingCompleted: boolean;
};

export type AppSnapshot = {
  settings: AppSettings;
  promptCount: number;
  modelConfigCount: number;
  activePromptId: string | null;
  activeModelConfigId: string | null;
  screenPermission: "granted" | "denied" | "unknown";
};

export type PhysicalRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type MonitorSummary = {
  id: string;
  name: string;
  bounds: PhysicalRect;
  scaleFactor: number;
  primary: boolean;
};

export type CaptureSessionSummary = {
  sessionId: string;
  monitors: MonitorSummary[];
};

export type AnalysisSnapshot = {
  runId: string;
  state: "submitting" | "streaming" | "completed" | "failed" | "cancelled";
  text: string;
  savedToHistory: boolean;
  error: AppError | null;
};

export type AnalysisEvent =
  | { type: "started"; runId: string }
  | { type: "delta"; runId: string; text: string }
  | { type: "completed"; runId: string; text: string; savedToHistory: boolean }
  | { type: "failed"; runId: string; error: AppError; savedToHistory: boolean }
  | { type: "cancelled"; runId: string };

export type ModelProtocol = "openai" | "anthropic" | "gemini";

export type ModelConfigInput = {
  id?: string;
  name: string;
  protocol: ModelProtocol;
  baseUrl: string;
  modelId: string;
  apiKey?: string;
  clearApiKey?: boolean;
};

export type ModelConfigSummary = {
  id: string;
  name: string;
  protocol: ModelProtocol;
  baseUrl: string;
  modelId: string;
  hasApiKey: boolean;
  testStatus: "untested" | "passed" | "failed";
  testedAt: string | null;
  testErrorCode: string | null;
  isActive: boolean;
};

export type ModelConnectionInput = {
  id?: string;
  protocol: ModelProtocol;
  baseUrl: string;
  modelId: string;
  apiKey?: string;
};

export type RemoteModel = { id: string; name: string };
export type ConnectionTestResult = {
  passed: boolean;
  latencyMs: number;
  error: AppError | null;
};

export type PromptPresetInput = { id?: string; name: string; body: string };
export type PromptPreset = {
  id: string;
  name: string;
  body: string;
  isBuiltin: boolean;
  isActive: boolean;
};

export type HistoryStatus = "success" | "failed";
export type HistoryQuery = {
  text?: string;
  promptName?: string;
  status?: HistoryStatus;
  cursor?: string;
  limit?: number;
};
export type HistoryListItem = {
  id: string;
  status: HistoryStatus;
  resultPreview: string | null;
  errorMessage: string | null;
  promptName: string;
  modelConfigName: string;
  modelId: string;
  startedAt: string;
  completedAt: string;
  hasImage: boolean;
};
export type HistoryPage = {
  items: HistoryListItem[];
  nextCursor: string | null;
};
export type HistoryEntryDetail = HistoryListItem & {
  resultText: string | null;
  errorCode: string | null;
  promptBody: string;
  protocol: string;
};

export const ipc = {
  getAppSnapshot: () => invoke<AppSnapshot>("get_app_snapshot"),
  beginCapture: () => invoke<CaptureSessionSummary>("begin_capture"),
  getCaptureFrame: (sessionId: string, monitorId: string) =>
    invoke<ArrayBuffer>("get_capture_frame", { sessionId, monitorId }),
  updateCaptureSelection: (sessionId: string, selection: PhysicalRect) =>
    invoke<void>("update_capture_selection", { sessionId, selection }),
  finishCapture: (sessionId: string, selection: PhysicalRect) =>
    invoke<{ runId: string }>("finish_capture", { sessionId, selection }),
  cancelCapture: (sessionId: string) =>
    invoke<void>("cancel_capture", { sessionId }),
  attachAnalysis: (runId: string, onEvent: Channel<AnalysisEvent>) =>
    invoke<AnalysisSnapshot>("attach_analysis", { runId, onEvent }),
  cancelAnalysis: (runId: string) => invoke<void>("cancel_analysis", { runId }),
  closeResult: (runId: string) => invoke<void>("close_result", { runId }),
  setResultAlwaysOnTop: (value: boolean) =>
    invoke<void>("set_result_always_on_top", { value }),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  listModelConfigs: () => invoke<ModelConfigSummary[]>("list_model_configs"),
  saveModelConfig: (input: ModelConfigInput) =>
    invoke<ModelConfigSummary>("save_model_config", { input }),
  deleteModelConfig: (id: string) =>
    invoke<void>("delete_model_config", { id }),
  setActiveModelConfig: (id: string) =>
    invoke<void>("set_active_model_config", { id }),
  listRemoteModels: (draft: ModelConnectionInput) =>
    invoke<RemoteModel[]>("list_remote_models", { draft }),
  testModelConfig: (draft: ModelConnectionInput) =>
    invoke<ConnectionTestResult>("test_model_config", { draft }),
  listPromptPresets: () => invoke<PromptPreset[]>("list_prompt_presets"),
  savePromptPreset: (input: PromptPresetInput) =>
    invoke<PromptPreset>("save_prompt_preset", { input }),
  duplicatePromptPreset: (id: string) =>
    invoke<PromptPreset>("duplicate_prompt_preset", { id }),
  deletePromptPreset: (id: string) =>
    invoke<void>("delete_prompt_preset", { id }),
  setActivePrompt: (id: string) => invoke<void>("set_active_prompt", { id }),
  queryHistory: (query: HistoryQuery) =>
    invoke<HistoryPage>("query_history", { query }),
  getHistoryEntry: (id: string) =>
    invoke<HistoryEntryDetail>("get_history_entry", { id }),
  getHistoryImage: (id: string, variant: "thumbnail" | "original") =>
    invoke<ArrayBuffer>("get_history_image", { id, variant }),
  resubmitHistory: (id: string) =>
    invoke<{ runId: string }>("resubmit_history", { id }),
  deleteHistoryEntry: (id: string) =>
    invoke<void>("delete_history_entry", { id }),
  clearHistory: () => invoke<{ deletedCount: number }>("clear_history"),
  setSaveHistory: (value: boolean) =>
    invoke<AppSettings>("set_save_history", { value }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  setCaptureShortcut: (shortcut: string) =>
    invoke<AppSettings>("set_capture_shortcut", { shortcut }),
  setAutostart: (value: boolean) =>
    invoke<AppSettings>("set_autostart", { value }),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
  openScreenPermissionSettings: () =>
    invoke<void>("open_screen_permission_settings"),
  exportSanitizedLogs: () =>
    invoke<{ exported: boolean }>("export_sanitized_logs"),
  quit: () => invoke<void>("quit_app"),
};
