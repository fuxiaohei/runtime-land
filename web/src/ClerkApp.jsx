import {
  ClerkLoaded,
  ClerkLoading,
  ClerkProvider,
  SignIn,
  SignUp,
} from "@clerk/clerk-react";
import { Navigate, Outlet, Route, Routes } from "react-router";
import AccountPage from "./pages/Account";
import LoadingPage from "./pages/Loading";
import NotFoundPage from "./pages/NotFound";
import ProjectOverviewPage from "./pages/ProjectOverview";
import ProjectSettingPage from "./pages/ProjectSetting";
import ProjectsPage from "./pages/Projects";
import AdminDomainsPage from "./pages/admin/Domains";
import AdminRegionsPage from "./pages/admin/Regions";
import AdminStatsPage from "./pages/admin/Stats";
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
        <Route path="/admin/stats" element={<AdminStatsPage />} />
        <Route path="/admin/regions" element={<AdminRegionsPage />} />
        <Route path="/admin/domains" element={<AdminDomainsPage />} />
        <Route path="/admin/storage" element={<AdminStoragePage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

function ClerkApp() {
  return (
    <ClerkProvider publishableKey={CLERK_KEY}>
      <ClerkLoading>
        <LoadingPage />
      </ClerkLoading>
      <ClerkLoaded>
        <App />
      </ClerkLoaded>
    </ClerkProvider>
  );
}

export default ClerkApp;
