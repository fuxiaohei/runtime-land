import React from "react";
import "bootstrap/dist/css/bootstrap.min.css";
import { Routes, Route, Outlet, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import ProjectsPage from "./pages/ProjectsPage";
import ProjectPage from "./pages/ProjectPage";
import NotFoundPage from "./pages/NotFoundPage";
import LoginEmailPage from "./pages/LoginEmailPage";
import {
  AuthProvider,
  RequireAuth,
  RequireUnauth,
  SignoutPage,
} from "./components/AuthContext";
import SettingsPage from "./pages/SettingsPage";
import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en";
import NewProjectPage from "./pages/NewProjectPage";
import SignupPage from "./pages/SignupPage";

TimeAgo.addDefaultLocale(en);

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route element={<Outlet />}>
          <Route path="/" element={<Navigate to="/projects" replace />} />
          {/** <Route
            path="/login"
            element={
              <RequireUnauth>
                <LoginPage />
              </RequireUnauth>
            }
          />*/}
          <Route
            path="/login-email"
            element={
              <RequireUnauth>
                <LoginEmailPage />
              </RequireUnauth>
            }
          />
          <Route
            path="/signup"
            element={
              <RequireUnauth>
                <SignupPage />
              </RequireUnauth>
            }
          />
          <Route
            path="/projects"
            element={
              <RequireAuth>
                <ProjectsPage />
              </RequireAuth>
            }
          />
          <Route
            path="/projects/:projectName"
            element={
              <RequireAuth>
                <ProjectPage />
              </RequireAuth>
            }
          />
          <Route
            path="/new"
            element={
              <RequireAuth>
                <NewProjectPage />
              </RequireAuth>
            }
          />

          <Route path="/signout" element={<SignoutPage />} />
          <Route
            path="/settings"
            element={
              <RequireAuth>
                <SettingsPage />
              </RequireAuth>
            }
          />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </AuthProvider>
  );
}

export default App;
