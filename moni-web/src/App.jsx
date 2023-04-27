import React from "react";
import "bootstrap/dist/css/bootstrap.min.css";
import { Routes, Route, Outlet, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import Dashboard from "./pages/Dashboard";
import ProjectPage from "./pages/ProjectPage";
import NotFoundPage from "./pages/NotFoundPage";
import LoginEmailPage from "./pages/LoginEmailPage";
import {
  AuthProvider,
  RequireAuth,
  RequireUnauth,
} from "./components/AuthContext";

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route element={<Outlet />}>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route
            path="/login"
            element={
              <RequireUnauth>
                <LoginPage />
              </RequireUnauth>
            }
          />
          <Route
            path="/login-email"
            element={
              <RequireUnauth>
                <LoginEmailPage />
              </RequireUnauth>
            }
          />
          <Route
            path="/dashboard"
            element={
              <RequireAuth>
                <Dashboard />
              </RequireAuth>
            }
          />
          <Route path="/project" element={<ProjectPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </AuthProvider>
  );
}

export default App;
