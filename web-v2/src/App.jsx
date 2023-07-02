import { AuthProvider } from "./contexts/Auth";
import { Routes, Route, Outlet, Navigate } from "react-router-dom";
import ProjectsPage from "./pages/Projects";
import NotFoundPage from "./pages/NotFound";

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route element={<Outlet />}>
          <Route path="/" element={<Navigate to="/projects" replace />} />
          <Route path="/projects" element={<ProjectsPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </AuthProvider>
  );
}

export default App;
