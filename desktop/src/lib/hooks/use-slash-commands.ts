import { useQuery } from "@tanstack/react-query";
import { agents } from "@/lib/api/endpoints";

export function useSlashCommands(provider: string | undefined) {
  return useQuery({
    queryKey: ["slash-commands", provider],
    queryFn: () => agents.getCommands(provider!),
    enabled: !!provider,
    staleTime: 5 * 60 * 1000,
  });
}
