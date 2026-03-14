import type { SandboxInfo } from "@/lib/api/types";
import SandboxCard from "./SandboxCard";

interface Props {
  sandboxes: SandboxInfo[];
}

export default function SandboxGrid({ sandboxes }: Props) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
      {sandboxes.map((sandbox) => (
        <SandboxCard key={sandbox.id} sandbox={sandbox} />
      ))}
    </div>
  );
}
