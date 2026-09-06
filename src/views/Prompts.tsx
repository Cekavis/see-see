import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { EmptyState } from "../components/EmptyState";
import { Field } from "../components/Field";
import { useNotifications } from "../components/Notifications";
import {
  getErrorMessage,
  ipc,
  type AppError,
  type PromptPreset,
  type PromptPresetInput,
} from "../ipc";
import { shortcutFromKeyboardEvent } from "./DesktopSettings";

export type PromptsApi = {
  listPromptPresets: () => Promise<PromptPreset[]>;
  savePromptPreset: (input: PromptPresetInput) => Promise<PromptPreset>;
  duplicatePromptPreset: (id: string) => Promise<PromptPreset>;
  deletePromptPreset: (id: string) => Promise<void>;
  setPromptShortcut: (
    id: string,
    shortcut: string | null,
  ) => Promise<PromptPreset>;
};
const empty = (): PromptPresetInput => ({ name: "", body: "" });

export function Prompts({ api = ipc }: { api?: PromptsApi }) {
  const notifications = useNotifications();
  const [prompts, setPrompts] = useState<PromptPreset[]>([]);
  const [editing, setEditing] = useState<PromptPresetInput | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<PromptPreset | null>(null);
  const [recording, setRecording] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [savingShortcut, setSavingShortcut] = useState<string | null>(null);
  const refresh = useCallback(
    () =>
      api
        .listPromptPresets()
        .then(setPrompts)
        .catch((v: AppError) => notifications.error(getErrorMessage(v))),
    [api, notifications],
  );
  useEffect(() => {
    void refresh();
  }, [refresh]);
  useEffect(() => {
    if (!recording) return;
    const onKey = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) return;
      if (
        event.key === "Escape" &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.shiftKey
      ) {
        setRecording(null);
        return;
      }
      const clear =
        ["Backspace", "Delete"].includes(event.key) &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.shiftKey;
      const value = shortcutFromKeyboardEvent(event);
      if (!clear && !value) return;
      setRecording(null);
      setSavingShortcut(recording);
      void api
        .setPromptShortcut(recording, clear ? null : value)
        .then((updated) => {
          setPrompts((current) =>
            current.map((prompt) =>
              prompt.id === updated.id ? updated : prompt,
            ),
          );
          notifications.success(clear ? "快捷键已清除" : "快捷键已保存并生效");
        })
        .catch((v: AppError) => notifications.error(getErrorMessage(v)))
        .finally(() => setSavingShortcut(null));
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, api, notifications]);
  if (editing)
    return (
      <section
        className="section-view prompts-view"
        aria-labelledby="prompt-editor-title"
      >
        <header className="settings-section__header">
          <h1 id="prompt-editor-title">
            {editing.id ? "编辑提示词" : "增加提示词"}
          </h1>
        </header>
        <div className="section-stack">
          <section className="settings-grid" aria-label="提示词编辑器">
            <Field label="提示词名称" htmlFor="prompt-name">
              <input
                id="prompt-name"
                maxLength={80}
                value={editing.name}
                onChange={(event) =>
                  setEditing({ ...editing, name: event.target.value })
                }
              />
            </Field>
            <Field
              label="提示词正文"
              htmlFor="prompt-body"
              hint={`截图提交时会保存提示词快照，后续编辑不影响已有记录。${editing.body.length}/20000`}
            >
              <textarea
                id="prompt-body"
                rows={12}
                maxLength={20000}
                value={editing.body}
                onChange={(event) =>
                  setEditing({ ...editing, body: event.target.value })
                }
              />
            </Field>
            <div className="button-row">
              <Button
                type="button"
                variant="primary"
                disabled={saving}
                onClick={() => {
                  setSaving(true);
                  void api
                    .savePromptPreset(editing)
                    .then(() => {
                      setEditing(null);
                      notifications.success("提示词已保存");
                      void refresh();
                    })
                    .catch((v: AppError) =>
                      notifications.error(getErrorMessage(v)),
                    )
                    .finally(() => setSaving(false));
                }}
              >
                {saving ? "保存中…" : "保存"}
              </Button>
              <Button
                type="button"
                disabled={saving}
                onClick={() => setEditing(null)}
              >
                取消
              </Button>
            </div>
          </section>
        </div>
      </section>
    );
  return (
    <section
      className="section-view prompts-view"
      aria-labelledby="prompts-title"
    >
      <header className="settings-section__header action-header">
        <h1 id="prompts-title">提示词</h1>
        <Button
          type="button"
          variant="primary"
          onClick={() => {
            setRecording(null);
            setEditing(empty());
          }}
        >
          增加提示词
        </Button>
      </header>
      <div className="section-stack">
        <section className="config-list" aria-label="提示词列表">
          {prompts.length === 0 ? (
            <EmptyState
              title="还没有提示词"
              description="新建一个提示词来定义模型需要输出的内容。"
            />
          ) : (
            prompts.map((prompt) => (
              <article className="config-card prompt-card" key={prompt.id}>
                <div>
                  <h3>{prompt.name}</h3>
                  <p className="prompt-card__preview">{prompt.body}</p>
                </div>
                <div className="prompt-card__actions">
                  <div className="button-row">
                    <Button
                      type="button"
                      onClick={() => {
                        setRecording(null);
                        setEditing({
                          id: prompt.id,
                          name: prompt.name,
                          body: prompt.body,
                        });
                      }}
                    >
                      编辑
                    </Button>
                    <Button
                      type="button"
                      onClick={() =>
                        void api
                          .duplicatePromptPreset(prompt.id)
                          .then(() => {
                            notifications.success("提示词已克隆");
                            void refresh();
                          })
                          .catch((v: AppError) =>
                            notifications.error(getErrorMessage(v)),
                          )
                      }
                    >
                      克隆
                    </Button>
                    <Button
                      type="button"
                      variant="danger"
                      onClick={() => {
                        setRecording(null);
                        setDeleteTarget(prompt);
                      }}
                    >
                      删除
                    </Button>
                  </div>
                  <Button
                    type="button"
                    className={`shortcut-recorder${!prompt.captureShortcut ? " shortcut-recorder--empty" : ""}`}
                    aria-label={`${prompt.name}截图快捷键`}
                    aria-pressed={recording === prompt.id}
                    disabled={savingShortcut !== null}
                    title="点击录制快捷键；Escape 取消，Backspace 或 Delete 清除"
                    onBlur={() =>
                      setRecording((current) =>
                        current === prompt.id ? null : current,
                      )
                    }
                    onClick={() => setRecording(prompt.id)}
                  >
                    {savingShortcut === prompt.id
                      ? "保存中…"
                      : recording === prompt.id
                        ? "请按新的快捷键…"
                        : prompt.captureShortcut || "点击设置快捷键"}
                  </Button>
                </div>
              </article>
            ))
          )}
        </section>
      </div>
      <ConfirmDialog
        open={Boolean(deleteTarget)}
        title="删除提示词？"
        description={`删除提示词“${deleteTarget?.name ?? ""}”？此操作不可撤销。`}
        confirmLabel="删除提示词"
        danger
        onCancel={() => setDeleteTarget(null)}
        onConfirm={() => {
          const target = deleteTarget;
          if (!target) return;
          setDeleteTarget(null);
          void api
            .deletePromptPreset(target.id)
            .then(() => {
              notifications.success("提示词已删除");
              void refresh();
            })
            .catch((v: AppError) => notifications.error(getErrorMessage(v)));
        }}
      />
    </section>
  );
}
