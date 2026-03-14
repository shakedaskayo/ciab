import { Outlet } from "react-router";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import { useUIStore } from "@/lib/stores/ui-store";

export default function AppShell() {
  const collapsed = useUIStore((s) => s.sidebarCollapsed);

  return (
    <div className="flex h-screen overflow-hidden bg-ciab-bg-primary">
      <Sidebar />
      <div
        className={`flex flex-col flex-1 min-w-0 transition-all duration-200 ml-0 ${
          collapsed ? "md:ml-[52px]" : "md:ml-56"
        }`}
      >
        <TopBar />
        <main className="flex-1 overflow-auto p-3 sm:p-5">
          <Outlet />
        </main>
      </div>
      {/* Subtle noise texture overlay */}
      <div className="noise-overlay" />
    </div>
  );
}
