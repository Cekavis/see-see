import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { Button } from "../components/Button";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { EmptyState } from "../components/EmptyState";
import { ErrorNotice } from "../components/ErrorNotice";
import { useNotifications } from "../components/Notifications";
import { ThinkingDisclosure } from "./Result";
import {
  getErrorMessage,
  ipc,
  type AppError,
  type HistoryEntryDetail,
  type HistoryListItem,
  type HistoryPage,
  type HistoryQuery,
  type ModelConfigSummary,
  type PromptPreset,
} from "../ipc";

export type HistoryApi = {
  queryHistory: (query: HistoryQuery) => Promise<HistoryPage>;
  getHistoryEntry: (id: string) => Promise<HistoryEntryDetail>;
  getHistoryImage: (
    id: string,
    variant: "thumbnail" | "original",
  ) => Promise<ArrayBuffer>;
  listModelConfigs: () => Promise<ModelConfigSummary[]>;
  listPromptPresets: () => Promise<PromptPreset[]>;
  resubmitHistory: (
    id: string,
    modelConfigId: string,
    promptConfigId: string,
  ) => Promise<{ runId: string }>;
  deleteHistoryEntry: (id: string) => Promise<void>;
  clearHistory: () => Promise<{ deletedCount: number }>;
  copyText: (text: string) => Promise<void>;
};

function defaultConfigurationId(
  options: { id: string; name: string; isActive: boolean }[],
  originalId: string | null,
  originalName: string,
) {
  return (
    options.find((option) => option.id === originalId)?.id ??
    options.find((option) => option.name === originalName)?.id ??
    options.find((option) => option.isActive)?.id ??
    options[0]?.id ??
    ""
  );
}

function useImage(
  api: HistoryApi,
  item: HistoryListItem | HistoryEntryDetail | null,
  variant: "thumbnail" | "original",
  onError: (message: string) => void,
) {
  const [url, setUrl] = useState<string>();
  useEffect(() => {
    if (!item?.hasImage || typeof URL.createObjectURL !== "function") return;
    let active = true;
    let objectUrl: string | undefined;
    void api
      .getHistoryImage(item.id, variant)
      .then((buffer) => {
        if (!active) return;
        objectUrl = URL.createObjectURL(
          new Blob([buffer], { type: "image/png" }),
        );
        setUrl(objectUrl);
      })
      .catch((value: unknown) => onError(getErrorMessage(value)));
    return () => {
      active = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [api, item, onError, variant]);
  return url;
}

function Thumbnail({ api, item }: { api: HistoryApi; item: HistoryListItem }) {
  const notifications = useNotifications();
  const url = useImage(api, item, "thumbnail", notifications.error);
  return url ? (
    <img className="history-item__thumbnail" src={url} alt="截图缩略图" />
  ) : (
    <div className="history-item__thumbnail history-item__thumbnail--empty" />
  );
}

export function History({ api = ipc }: { api?: HistoryApi }) {
  const notifications = useNotifications();
  const rootRef = useRef<HTMLElement>(null);
  const listScrollTop = useRef(0);
  const restoreListScroll = useRef(false);
  const [items, setItems] = useState<HistoryListItem[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [promptName, setPromptName] = useState("");
  const [status, setStatus] = useState<"" | "success" | "failed">("");
  const [detail, setDetail] = useState<HistoryEntryDetail | null>(null);
  const [modelConfigs, setModelConfigs] = useState<ModelConfigSummary[]>([]);
  const [promptPresets, setPromptPresets] = useState<PromptPreset[]>([]);
  const [selectedModelConfigId, setSelectedModelConfigId] = useState("");
  const [selectedPromptConfigId, setSelectedPromptConfigId] = useState("");
  const [configurationsLoading, setConfigurationsLoading] = useState(false);
  const [resubmitting, setResubmitting] = useState(false);
  const [confirmation, setConfirmation] = useState<
    { kind: "entry"; item: HistoryListItem } | { kind: "all" } | null
  >(null);
  const imageUrl = useImage(api, detail, "original", notifications.error);

  useLayoutEffect(() => {
    const scrollContainer =
      rootRef.current?.closest<HTMLElement>(".settings-content");
    if (detail) {
      if (scrollContainer) scrollContainer.scrollTop = 0;
      return;
    }
    if (!restoreListScroll.current) return;
    if (scrollContainer) scrollContainer.scrollTop = listScrollTop.current;
    restoreListScroll.current = false;
  }, [detail]);

  useEffect(() => {
    if (!detail) return;
    let active = true;
    void Promise.all([api.listModelConfigs(), api.listPromptPresets()])
      .then(([models, prompts]) => {
        if (!active) return;
        setModelConfigs(models);
        setPromptPresets(prompts);
        setSelectedModelConfigId(
          defaultConfigurationId(
            models,
            detail.modelConfigId,
            detail.modelConfigName,
          ),
        );
        setSelectedPromptConfigId(
          defaultConfigurationId(
            prompts,
            detail.promptConfigId,
            detail.promptName,
          ),
        );
      })
      .catch((value: AppError) => {
        if (active) notifications.error(value.message);
      })
      .finally(() => {
        if (active) setConfigurationsLoading(false);
      });
    return () => {
      active = false;
    };
  }, [api, detail, notifications]);

  const query = useCallback(
    (cursor?: string, append = false) => {
      const value: HistoryQuery = {
        text: text || undefined,
        promptName: promptName || undefined,
        status: status || undefined,
        cursor,
      };
      return api
        .queryHistory(value)
        .then((page) => {
          setItems((current) =>
            append ? [...current, ...page.items] : page.items,
          );
          setNextCursor(page.nextCursor);
        })
        .catch((value: AppError) => notifications.error(value.message));
    },
    [api, notifications, promptName, status, text],
  );

  useEffect(() => {
    function load() {
      void api
        .queryHistory({})
        .then((page) => {
          setItems(page.items);
          setNextCursor(page.nextCursor);
        })
        .catch((value: AppError) =>
          notifications.error(value.message, {
            action: { label: "重试", onClick: load },
          }),
        );
    }
    load();
  }, [api, notifications]);

  const hasFilters = Boolean(text || promptName || status);
  if (detail) {
    return (
      <section
        ref={rootRef}
        className="section-view history-view"
        aria-labelledby="history-detail-title"
      >
        <header className="settings-section__header history-detail-header">
          <Button
            onClick={() => {
              restoreListScroll.current = true;
              setDetail(null);
            }}
          >
            返回历史记录
          </Button>
          <h1 id="history-detail-title">历史详情</h1>
        </header>
        <div className="history-detail-layout">
          <section className="history-detail" aria-labelledby="detail-heading">
            <h2 id="detail-heading">
              {detail.status === "success" ? "识别结果" : "失败详情"}
            </h2>
            {imageUrl && (
              <img
                className="history-detail__image"
                src={imageUrl}
                alt="原始截图"
              />
            )}
            <dl>
              <dt>提示词</dt>
              <dd>{detail.promptName}</dd>
              <dt>模型</dt>
              <dd>
                {detail.modelConfigName} · {detail.modelId}
              </dd>
            </dl>
            <ThinkingDisclosure text={detail.thinkingText} />
            {detail.status === "success" ? (
              <pre className="result-view__text">{detail.resultText}</pre>
            ) : (
              <ErrorNotice
                message={detail.errorMessage ?? detail.errorCode ?? "分析失败"}
              />
            )}
            <div className="history-resubmit-config">
              <label>
                模型配置
                <select
                  aria-label="模型配置"
                  value={selectedModelConfigId}
                  disabled={configurationsLoading || modelConfigs.length === 0}
                  onChange={(event) =>
                    setSelectedModelConfigId(event.target.value)
                  }
                >
                  {modelConfigs.length === 0 && (
                    <option value="">
                      {configurationsLoading
                        ? "正在加载模型配置"
                        : "没有可用模型配置"}
                    </option>
                  )}
                  {modelConfigs.map((model) => (
                    <option key={model.id} value={model.id}>
                      {model.name} · {model.modelId}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                提示词配置
                <select
                  aria-label="提示词配置"
                  value={selectedPromptConfigId}
                  disabled={configurationsLoading || promptPresets.length === 0}
                  onChange={(event) =>
                    setSelectedPromptConfigId(event.target.value)
                  }
                >
                  {promptPresets.length === 0 && (
                    <option value="">
                      {configurationsLoading
                        ? "正在加载提示词配置"
                        : "没有可用提示词配置"}
                    </option>
                  )}
                  {promptPresets.map((prompt) => (
                    <option key={prompt.id} value={prompt.id}>
                      {prompt.name}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            <div className="button-row">
              <Button
                disabled={!detail.resultText}
                onClick={() => {
                  if (!detail.resultText) return;
                  notifications.clear();
                  void api
                    .copyText(detail.resultText)
                    .then(() => notifications.success("结果已复制"))
                    .catch((value: AppError) =>
                      notifications.error(value.message),
                    );
                }}
              >
                复制结果
              </Button>
              <Button
                variant="primary"
                disabled={
                  !detail.hasImage ||
                  configurationsLoading ||
                  resubmitting ||
                  !selectedModelConfigId ||
                  !selectedPromptConfigId
                }
                onClick={() => {
                  notifications.clear();
                  setResubmitting(true);
                  void api
                    .resubmitHistory(
                      detail.id,
                      selectedModelConfigId,
                      selectedPromptConfigId,
                    )
                    .then(() => notifications.success("已重新提交"))
                    .catch((value: AppError) =>
                      notifications.error(value.message),
                    )
                    .finally(() => setResubmitting(false));
                }}
              >
                重新选择配置提交
              </Button>
            </div>
          </section>
        </div>
      </section>
    );
  }

  return (
    <section
      ref={rootRef}
      className="section-view history-view"
      aria-labelledby="history-title"
    >
      <header className="settings-section__header history-header">
        <h1 id="history-title">历史记录</h1>
        <Button
          variant="danger"
          onClick={() => setConfirmation({ kind: "all" })}
        >
          清空全部历史
        </Button>
      </header>
      <section className="history-filters" aria-label="历史筛选">
        <label>
          搜索结果
          <input
            aria-label="搜索结果"
            value={text}
            onChange={(event) => setText(event.target.value)}
          />
        </label>
        <label>
          提示词
          <input
            aria-label="提示词"
            value={promptName}
            onChange={(event) => setPromptName(event.target.value)}
          />
        </label>
        <label>
          状态
          <select
            aria-label="状态"
            value={status}
            onChange={(event) => setStatus(event.target.value as typeof status)}
          >
            <option value="">全部</option>
            <option value="success">成功</option>
            <option value="failed">失败</option>
          </select>
        </label>
        <Button variant="primary" onClick={() => void query()}>
          搜索
        </Button>
      </section>
      <div className="history-layout">
        <section className="history-list" aria-label="历史列表">
          {items.length === 0 ? (
            <EmptyState
              title={hasFilters ? "没有匹配的历史记录" : "没有历史记录"}
            />
          ) : (
            items.map((item) => (
              <article className="history-item" key={item.id}>
                <Thumbnail api={api} item={item} />
                <div className="history-item__content">
                  <p className="history-item__meta">
                    {item.status === "success" ? "成功" : "失败"} ·{" "}
                    {item.promptName} ·{" "}
                    {new Date(item.startedAt).toLocaleString()}
                  </p>
                  <pre className="history-item__summary">
                    {item.resultPreview ?? item.errorMessage ?? "无结果"}
                  </pre>
                  <div className="button-row">
                    <Button
                      onClick={() => {
                        const scrollContainer =
                          rootRef.current?.closest<HTMLElement>(
                            ".settings-content",
                          );
                        listScrollTop.current = scrollContainer?.scrollTop ?? 0;
                        void api
                          .getHistoryEntry(item.id)
                          .then((entry) => {
                            setModelConfigs([]);
                            setPromptPresets([]);
                            setSelectedModelConfigId("");
                            setSelectedPromptConfigId("");
                            setConfigurationsLoading(true);
                            setDetail(entry);
                          })
                          .catch((value: AppError) =>
                            notifications.error(value.message),
                          );
                      }}
                    >
                      查看详情
                    </Button>
                    <Button
                      variant="danger"
                      onClick={() => setConfirmation({ kind: "entry", item })}
                    >
                      删除记录
                    </Button>
                  </div>
                </div>
              </article>
            ))
          )}
          {nextCursor && (
            <Button onClick={() => void query(nextCursor, true)}>
              加载更多
            </Button>
          )}
        </section>
      </div>
      <ConfirmDialog
        open={Boolean(confirmation)}
        title={
          confirmation?.kind === "all" ? "清空全部历史？" : "删除历史记录？"
        }
        description={
          confirmation?.kind === "all"
            ? "将删除全部历史记录和截图，此操作不可撤销。"
            : "将删除这条历史记录和截图，此操作不可撤销。"
        }
        confirmLabel={confirmation?.kind === "all" ? "确认清空" : "删除记录"}
        danger
        onCancel={() => setConfirmation(null)}
        onConfirm={() => {
          const target = confirmation;
          if (!target) return;
          setConfirmation(null);
          if (target.kind === "all") {
            void api
              .clearHistory()
              .then(() => {
                setDetail(null);
                notifications.success("历史记录已清空");
                void query();
              })
              .catch((value: AppError) => notifications.error(value.message));
            return;
          }
          void api
            .deleteHistoryEntry(target.item.id)
            .then(() => {
              notifications.success("历史记录已删除");
              void query();
            })
            .catch((value: AppError) => notifications.error(value.message));
        }}
      />
    </section>
  );
}
