import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { EmptyState } from "../components/EmptyState";
import { ErrorNotice } from "../components/ErrorNotice";
import {
  ipc,
  type AppError,
  type HistoryEntryDetail,
  type HistoryListItem,
  type HistoryPage,
  type HistoryQuery,
} from "../ipc";

export type HistoryApi = {
  queryHistory: (query: HistoryQuery) => Promise<HistoryPage>;
  getHistoryEntry: (id: string) => Promise<HistoryEntryDetail>;
  getHistoryImage: (
    id: string,
    variant: "thumbnail" | "original",
  ) => Promise<ArrayBuffer>;
  resubmitHistory: (id: string) => Promise<{ runId: string }>;
  deleteHistoryEntry: (id: string) => Promise<void>;
  clearHistory: () => Promise<{ deletedCount: number }>;
  copyText: (text: string) => Promise<void>;
};

function useImage(
  api: HistoryApi,
  item: HistoryListItem | HistoryEntryDetail | null,
  variant: "thumbnail" | "original",
) {
  const [url, setUrl] = useState<string>();
  useEffect(() => {
    if (!item?.hasImage || typeof URL.createObjectURL !== "function") return;
    let active = true;
    let objectUrl: string | undefined;
    void api.getHistoryImage(item.id, variant).then((buffer) => {
      if (!active) return;
      objectUrl = URL.createObjectURL(
        new Blob([buffer], { type: "image/png" }),
      );
      setUrl(objectUrl);
    });
    return () => {
      active = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [api, item, variant]);
  return url;
}

function Thumbnail({ api, item }: { api: HistoryApi; item: HistoryListItem }) {
  const url = useImage(api, item, "thumbnail");
  return url ? (
    <img className="history-item__thumbnail" src={url} alt="截图缩略图" />
  ) : (
    <div className="history-item__thumbnail history-item__thumbnail--empty" />
  );
}

export function History({ api = ipc }: { api?: HistoryApi }) {
  const [items, setItems] = useState<HistoryListItem[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [promptName, setPromptName] = useState("");
  const [status, setStatus] = useState<"" | "success" | "failed">("");
  const [detail, setDetail] = useState<HistoryEntryDetail | null>(null);
  const [error, setError] = useState<string>();
  const [confirmation, setConfirmation] = useState<
    { kind: "entry"; item: HistoryListItem } | { kind: "all" } | null
  >(null);
  const imageUrl = useImage(api, detail, "original");

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
        .catch((value: AppError) => setError(value.message));
    },
    [api, promptName, status, text],
  );

  useEffect(() => {
    void api
      .queryHistory({})
      .then((page) => {
        setItems(page.items);
        setNextCursor(page.nextCursor);
      })
      .catch((value: AppError) => setError(value.message));
  }, [api]);

  const hasFilters = Boolean(text || promptName || status);
  return (
    <section
      className="section-view history-view"
      aria-labelledby="history-title"
    >
      <header className="settings-section__header history-header">
        <div>
          <h1 id="history-title">历史记录</h1>
          <p>截图、提示词快照和模型结果仅保存在本机。</p>
        </div>
        <Button
          variant="danger"
          onClick={() => setConfirmation({ kind: "all" })}
        >
          清空全部历史
        </Button>
      </header>
      {error && <ErrorNotice message={error} />}
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
              <article
                className={`history-item ${detail?.id === item.id ? "history-item--selected" : ""}`}
                key={item.id}
              >
                <Thumbnail api={api} item={item} />
                <div className="history-item__content">
                  <p className="history-item__meta">
                    {item.status === "success" ? "成功" : "失败"} ·{" "}
                    {item.promptName} ·{" "}
                    {new Date(item.startedAt).toLocaleString()}
                  </p>
                  <p>{item.resultPreview ?? item.errorMessage ?? "无结果"}</p>
                  <div className="button-row">
                    <Button
                      onClick={() =>
                        void api
                          .getHistoryEntry(item.id)
                          .then(setDetail)
                          .catch((value: AppError) => setError(value.message))
                      }
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
        <aside className="history-detail" aria-label="历史详情">
          {!detail ? (
            <EmptyState title="选择一条记录查看详情" />
          ) : (
            <>
              <h2>{detail.status === "success" ? "识别结果" : "失败详情"}</h2>
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
              {detail.status === "success" ? (
                <pre className="result-view__text">{detail.resultText}</pre>
              ) : (
                <ErrorNotice
                  message={
                    detail.errorMessage ?? detail.errorCode ?? "分析失败"
                  }
                />
              )}
              <div className="button-row">
                <Button
                  disabled={!detail.resultText}
                  onClick={() =>
                    detail.resultText && void api.copyText(detail.resultText)
                  }
                >
                  复制结果
                </Button>
                <Button
                  variant="primary"
                  disabled={!detail.hasImage}
                  onClick={() =>
                    void api
                      .resubmitHistory(detail.id)
                      .catch((value: AppError) => setError(value.message))
                  }
                >
                  使用当前配置再次提交
                </Button>
              </div>
            </>
          )}
        </aside>
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
                void query();
              })
              .catch((value: AppError) => setError(value.message));
            return;
          }
          void api
            .deleteHistoryEntry(target.item.id)
            .then(() => {
              if (detail?.id === target.item.id) setDetail(null);
              void query();
            })
            .catch((value: AppError) => setError(value.message));
        }}
      />
    </section>
  );
}
