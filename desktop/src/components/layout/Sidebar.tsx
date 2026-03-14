import { useEffect } from "react";
import { NavLink, useLocation } from "react-router";
import {
  LayoutGrid,
  Container,
  Layers,
  Zap,
  KeyRound,
  Globe,
  MessageSquare,
  Settings,
  ChevronsLeft,
  ChevronsRight,
  X,
} from "lucide-react";
import { useUIStore } from "@/lib/stores/ui-store";
import CiabLogo from "@/components/shared/CiabLogo";

const navItems = [
  { to: "/", icon: LayoutGrid, label: "Dashboard", end: true },
  { to: "/sandboxes", icon: Container, label: "Sandboxes" },
  { to: "/workspaces", icon: Layers, label: "Workspaces" },
  { to: "/skills", icon: Zap, label: "Skills" },
  { to: "/credentials", icon: KeyRound, label: "Credentials" },
  { to: "/gateway", icon: Globe, label: "Gateway" },
  { to: "/channels", icon: MessageSquare, label: "Channels" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const setSidebarOpen = useUIStore((s) => s.setSidebarOpen);
  const location = useLocation();

  // Close mobile sidebar on route change
  useEffect(() => {
    setSidebarOpen(false);
  }, [location.pathname, setSidebarOpen]);

  return (
    <>
      {/* Mobile backdrop */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 bg-black/60 backdrop-blur-sm z-30 md:hidden animate-fade-in"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <aside
        className={`fixed left-0 top-0 h-full bg-ciab-bg-secondary border-r border-ciab-border
          flex flex-col transition-all duration-200
          ${collapsed ? "w-[52px]" : "w-56"}
          max-md:w-64 max-md:z-40
          ${sidebarOpen ? "max-md:translate-x-0" : "max-md:-translate-x-full"}
          md:z-20
        `}
      >
        {/* Logo area */}
        <div
          className={`flex items-center border-b border-ciab-border ${
            collapsed
              ? "justify-center h-14 px-0 max-md:justify-between max-md:px-4"
              : "gap-3 px-4 py-4"
          }`}
        >
          <div className="flex items-center gap-3">
            <div
              className={`flex-shrink-0 rounded-lg ${
                collapsed ? "max-md:p-1 max-md:bg-gradient-to-br max-md:from-ciab-copper/15 max-md:to-ciab-copper/5 max-md:border max-md:border-ciab-copper/20" : "p-1 bg-gradient-to-br from-ciab-copper/15 to-ciab-copper/5 border border-ciab-copper/20"
              }`}
            >
              <CiabLogo size={collapsed ? 28 : 36} />
            </div>
            {(!collapsed || sidebarOpen) && (
              <div className={`flex flex-col min-w-0 ${collapsed ? "md:hidden" : ""}`}>
                <span className="font-display text-[17px] font-bold tracking-tight text-ciab-text-primary leading-tight">
                  CIAB
                </span>
                <span className="text-[8px] font-mono text-ciab-copper/70 uppercase tracking-[0.2em] leading-tight">
                  Claude In A Box
                </span>
              </div>
            )}
          </div>
          {/* Mobile close button */}
          <button
            onClick={() => setSidebarOpen(false)}
            className="md:hidden p-1 rounded text-ciab-text-muted hover:text-ciab-text-primary"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-3 px-2 space-y-0.5 overflow-y-auto">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              className={({ isActive }) =>
                `group flex items-center gap-2.5 px-2.5 py-2 rounded-md text-[13px] font-medium transition-all duration-100
                ${isActive
                  ? "bg-ciab-copper/10 text-ciab-copper shadow-sm shadow-ciab-copper/5"
                  : "text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover"
                }
                ${collapsed ? "md:justify-center md:px-0 md:mx-0" : ""}
                max-md:py-2.5 max-md:text-[14px]`
              }
            >
              {({ isActive }) => (
                <>
                  <item.icon
                    className={`w-[18px] h-[18px] flex-shrink-0 transition-colors ${
                      isActive ? "text-ciab-copper" : "text-ciab-text-muted group-hover:text-ciab-text-secondary"
                    }`}
                    strokeWidth={isActive ? 2 : 1.5}
                  />
                  {/* On mobile always show labels; on desktop respect collapsed */}
                  <span className={collapsed ? "md:hidden" : ""}>{item.label}</span>
                </>
              )}
            </NavLink>
          ))}
        </nav>

        {/* Collapse toggle — desktop only */}
        <div className="p-1.5 border-t border-ciab-border hidden md:block">
          <button
            onClick={toggleSidebar}
            className="btn-ghost w-full flex items-center justify-center py-1.5"
          >
            {collapsed ? (
              <ChevronsRight className="w-4 h-4" />
            ) : (
              <ChevronsLeft className="w-4 h-4" />
            )}
          </button>
        </div>
      </aside>
    </>
  );
}
