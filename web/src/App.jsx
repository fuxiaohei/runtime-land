import { AuthProvider } from "./contexts/Auth";
import { Routes, Route, Outlet, Navigate } from "react-router-dom";
import ProjectsPage from "./pages/Projects";
import NotFoundPage from "./pages/NotFound";
import ProjectOverviewPage from "./pages/ProjectOverview";
import LoginPage from "./pages/Login";
import SignUpPage from "./pages/SignUp";

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route element={<Outlet />}>
          <Route path="/" element={<Navigate to="/projects" replace />} />
          <Route path="/projects" element={<ProjectsPage />} />
          <Route
            path="/projects/:projectName/overview"
            element={<ProjectOverviewPage />}
          />
          <Route path="/login" element={<LoginPage />} />
          <Route path="/signup" element={<SignUpPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </AuthProvider>
  );
}

export default App;
