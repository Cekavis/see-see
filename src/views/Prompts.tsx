import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { ErrorNotice } from "../components/ErrorNotice";
import { Field } from "../components/Field";
import {
  ipc,
  type AppError,
  type PromptPreset,
  type PromptPresetInput,
} from "../ipc";

export type PromptsApi = {
  listPromptPresets: () => Promise<PromptPreset[]>;
  savePromptPreset: (input: PromptPresetInput) => Promise<PromptPreset>;
  duplicatePromptPreset: (id: string) => Promise<PromptPreset>;
  deletePromptPreset: (id: string) => Promise<void>;
  setActivePrompt: (id: string) => Promise<void>;
};

const empty = (): PromptPresetInput => ({ name: "", body: "" });

export function Prompts({ api = ipc }: { api?: PromptsApi }) {
  const [prompts, setPrompts] = useState<PromptPreset[]>([]);
  const [form, setForm] = useState<PromptPresetInput>(empty);
  const [error, setError] = useState<string>();
  const refresh = useCallback(
    () =>
      api
        .listPromptPresets()
        .then(setPrompts)
        .catch((value: AppError) => setError(value.message)),
    [api],
  );
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <main className="prompts-view app-shell">
      <header className="page-header">
        <h1>提示词</h1>
        <p>
          当前提示词会在截图提交时固定为快照，后续编辑不会改变正在进行或已有的记录。
        </p>
      </header>
      {error && <ErrorNotice message={error} />}
      <section className="settings-grid" aria-label="提示词编辑器">
        <Field label="提示词名称" htmlFor="prompt-name">
          <input
            id="prompt-name"
            maxLength={80}
            value={form.name}
            onChange={(event) =>
              setForm((current) => ({ ...current, name: event.target.value }))
            }
          />
        </Field>
        <Field
          label="提示词正文"
          htmlFor="prompt-body"
          hint={`${form.body.length}/20000`}
        >
          <textarea
            id="prompt-body"
            rows={12}
            maxLength={20_000}
            value={form.body}
            onChange={(event) =>
              setForm((current) => ({ ...current, body: event.target.value }))
            }
          />
        </Field>
        <div className="button-row">
          <Button
            type="button"
            variant="primary"
            onClick={() => {
              setError(undefined);
              void api
                .savePromptPreset(form)
                .then(() => {
                  setForm(empty());
                  void refresh();
                })
                .catch((value: AppError) => setError(value.message));
            }}
          >
            保存提示词
          </Button>
          {form.id && (
            <Button type="button" onClick={() => setForm(empty())}>
              新建提示词
            </Button>
          )}
        </div>
      </section>
      <section className="config-list" aria-label="提示词列表">
        <h2>已有提示词</h2>
        {prompts.length === 0 ? (
          <EmptyState
            title="还没有提示词"
            description="新建一个提示词来定义模型需要输出的内容。"
          />
        ) : (
          prompts.map((prompt) => (
            <article className="config-card prompt-card" key={prompt.id}>
              <div>
                <h3>
                  {prompt.name}
                  {prompt.isActive ? " · 当前" : ""}
                </h3>
                <p className="prompt-card__preview">{prompt.body}</p>
              </div>
              <div className="button-row">
                <Button
                  type="button"
                  onClick={() =>
                    setForm({
                      id: prompt.id,
                      name: prompt.name,
                      body: prompt.body,
                    })
                  }
                >
                  编辑
                </Button>
                <Button
                  type="button"
                  onClick={() =>
                    void api
                      .duplicatePromptPreset(prompt.id)
                      .then(refresh)
                      .catch((value: AppError) => setError(value.message))
                  }
                >
                  复制
                </Button>
                <Button
                  type="button"
                  disabled={prompt.isActive}
                  onClick={() =>
                    void api
                      .setActivePrompt(prompt.id)
                      .then(refresh)
                      .catch((value: AppError) => setError(value.message))
                  }
                >
                  设为当前
                </Button>
                <Button
                  type="button"
                  variant="danger"
                  onClick={() => {
                    if (window.confirm(`删除提示词“${prompt.name}”？`)) {
                      void api
                        .deletePromptPreset(prompt.id)
                        .then(() => {
                          if (form.id === prompt.id) setForm(empty());
                          void refresh();
                        })
                        .catch((value: AppError) => setError(value.message));
                    }
                  }}
                >
                  删除
                </Button>
              </div>
            </article>
          ))
        )}
      </section>
    </main>
  );
}
