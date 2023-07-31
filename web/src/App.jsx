import { AuthProvider } from "./contexts/Auth";
import { Routes, Route, Outlet, Navigate } from "react-router-dom";
import ProjectsPage from "./pages/Projects";
import NotFoundPage from "./pages/NotFound";
import ProjectOverviewPage from "./pages/ProjectOverview";
import {
  ClerkProvider,
  ClerkLoading,
  ClerkLoaded,
  SignIn,
  SignUp,
} from "@clerk/clerk-react";
import AccountPage from "./pages/Account";
import AdminPage from "./pages/admin/Admin";

const clerkPubKey = "pk_test_cGV0LW1vb3NlLTc1LmNsZXJrLmFjY291bnRzLmRldiQ";

function AppRoutes() {
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
        <Route
          path="/projects"
          element={
            <AuthProvider>
              <ProjectsPage />
            </AuthProvider>
          }
        />
        <Route
          path="/projects/:projectName/overview"
          element={
            <AuthProvider>
              <ProjectOverviewPage />
            </AuthProvider>
          }
        />
        <Route
          path="/account"
          element={
            <AuthProvider>
              <AccountPage />
            </AuthProvider>
          }
        />
        <Route
          path="/admin"
          element={
            <AuthProvider>
              <AdminPage />
            </AuthProvider>
          }
        />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <ClerkProvider publishableKey={clerkPubKey}>
      <ClerkLoading>
        <div>Clerk is loading...</div>
      </ClerkLoading>
      <ClerkLoaded>
        <AppRoutes />
      </ClerkLoaded>
    </ClerkProvider>
  );
}

export default App;
