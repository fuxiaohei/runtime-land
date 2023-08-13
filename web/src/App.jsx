import { Routes, Route, Outlet, Navigate } from "react-router";
import { SignIn, SignUp } from "@clerk/clerk-react";
import ProjectsPage from "./pages/Projects";
import ProjectOverviewPage from "./pages/ProjectOverview";
import ProjectSettingPage from "./pages/ProjectSetting";
import AccountPage from "./pages/Account";
import NotFoundPage from "./pages/NotFound";
import AdminPage from "./pages/admin/Admin";
import AdminStatsPage from "./pages/admin/Stats";
import AdminRegionsPage from "./pages/admin/Regions";
import AdminDomainsPage from "./pages/admin/Domains";
import AdminStoragePage from "./pages/admin/Storage";

function App() {
  return (
    <Routes>
      <Route element={<Outlet />}>
        <Route
          path="/sign-in/*"
          element={<SignIn routing="path" path="/sign-in" />}
        />
        <Route
          path="/sign-up/*"
          element={<SignUp routing="path" path="/sign-up" />}
        />
        <Route path="/" element={<Navigate to="/projects" replace />} />
        <Route path="/projects" element={<ProjectsPage />} />
        <Route
          path="/projects/:name/overview"
          element={<ProjectOverviewPage />}
        />
        <Route
          path="/projects/:name/setting"
          element={<ProjectSettingPage />}
        />
        <Route path="/account" element={<AccountPage />} />
        <Route path="/admin" element={<AdminPage />} />
        <Route path="/admin/stats" element={<AdminStatsPage />} />
        <Route path="/admin/regions" element={<AdminRegionsPage />} />
        <Route path="/admin/domains" element={<AdminDomainsPage />} />
        <Route path="/admin/storage" element={<AdminStoragePage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

export default App;
