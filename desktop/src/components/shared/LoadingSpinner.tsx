import { Loader2 } from "lucide-react";

interface Props {
  size?: number;
  className?: string;
}

export default function LoadingSpinner({ size = 24, className = "" }: Props) {
  return (
    <Loader2
      size={size}
      className={`animate-spin text-ciab-copper ${className}`}
    />
  );
}
