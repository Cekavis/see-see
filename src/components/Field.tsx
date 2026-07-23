import type { PropsWithChildren, ReactNode } from "react";

type Props = PropsWithChildren<{
  label: string;
  htmlFor: string;
  hint?: ReactNode;
  error?: string;
}>;

export function Field({ label, htmlFor, hint, error, children }: Props) {
  return (
    <div className="field">
      <label htmlFor={htmlFor}>{label}</label>
      {children}
      {hint && <div className="field__hint">{hint}</div>}
      {error && (
        <div className="field__error" role="alert">
          {error}
        </div>
      )}
    </div>
  );
}
