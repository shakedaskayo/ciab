import { create } from "zustand";

interface UIState {
  sidebarCollapsed: boolean;
  sidebarOpen: boolean; // mobile: overlay open/closed
  activeSandboxId: string | null;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setActiveSandboxId: (id: string | null) => void;
}

export const useUIStore = create<UIState>()((set) => ({
  sidebarCollapsed: false,
  sidebarOpen: false,
  activeSandboxId: null,
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  setActiveSandboxId: (id) => set({ activeSandboxId: id }),
}));
