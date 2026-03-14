import { Routes, Route } from "react-router";
import AppShell from "./components/layout/AppShell";
import Dashboard from "./pages/Dashboard";
import SandboxList from "./pages/SandboxList";
import SandboxDetail from "./pages/SandboxDetail";
import Credentials from "./pages/Credentials";
import WorkspaceList from "./pages/WorkspaceList";
import WorkspaceDetail from "./pages/WorkspaceDetail";
import Settings from "./pages/Settings";
import SkillsCatalog from "./pages/SkillsCatalog";
import Gateway from "./pages/Gateway";
import ChannelList from "./pages/ChannelList";
import ChannelDetail from "./pages/ChannelDetail";

export default function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Dashboard />} />
        <Route path="sandboxes" element={<SandboxList />} />
        <Route path="sandboxes/:id" element={<SandboxDetail />} />
        <Route path="workspaces" element={<WorkspaceList />} />
        <Route path="workspaces/:id" element={<WorkspaceDetail />} />
        <Route path="skills" element={<SkillsCatalog />} />
        <Route path="credentials" element={<Credentials />} />
        <Route path="gateway" element={<Gateway />} />
        <Route path="channels" element={<ChannelList />} />
        <Route path="channels/:id" element={<ChannelDetail />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}
