import type { LucideIcon } from "lucide-react";

interface Props {
  icon: LucideIcon;
  title: string;
  description: string;
  action?: React.ReactNode;
}

export default function EmptyState({
  icon: Icon,
  title,
  description,
  action,
}: Props) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="w-12 h-12 rounded-lg bg-ciab-bg-elevated flex items-center justify-center mb-3">
        <Icon className="w-6 h-6 text-ciab-text-muted" strokeWidth={1.5} />
      </div>
      <h3 className="text-sm font-medium text-ciab-text-primary mb-1">
        {title}
      </h3>
      <p className="text-xs text-ciab-text-muted max-w-sm mb-4 leading-relaxed">
        {description}
      </p>
      {action}
    </div>
  );
}
