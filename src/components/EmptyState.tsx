import type { ReactNode } from "react";

export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <section className="empty-state">
      <p>{title}</p>
      {description && <p className="field__hint">{description}</p>}
      {action}
    </section>
  );
}
