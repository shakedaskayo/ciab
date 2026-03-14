import { useQuery } from "@tanstack/react-query";
import { skills } from "@/lib/api/endpoints";

export function useSkillSearch(query: string, limit = 20) {
  return useQuery({
    queryKey: ["skills-search", query, limit],
    queryFn: () => skills.search(query, limit),
    enabled: query.length >= 2,
    staleTime: 60_000,
    placeholderData: (prev) => prev,
  });
}

export function useTrendingSkills() {
  return useQuery({
    queryKey: ["skills-trending"],
    queryFn: () => skills.trending(),
    staleTime: 5 * 60_000, // cache for 5 min
  });
}

export function useSkillMetadata(source: string | undefined, skillId?: string) {
  return useQuery({
    queryKey: ["skill-metadata", source, skillId],
    queryFn: () => skills.metadata(source!, skillId),
    enabled: !!source,
    staleTime: 5 * 60_000, // cache for 5 min
  });
}
