import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { HelmetProvider } from "react-helmet-async";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ClerkProvider, ClerkLoading, ClerkLoaded } from "@clerk/clerk-react";

import "./style.scss";
import App from "./App";
import LoadingPage from "./pages/Loading";

import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en.json";
TimeAgo.addDefaultLocale(en);

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <BrowserRouter>
      <HelmetProvider>
        <QueryClientProvider client={new QueryClient()}>
          <ClerkProvider publishableKey={CLERK_KEY}>
            <ClerkLoading>
              <LoadingPage />
            </ClerkLoading>
            <ClerkLoaded>
              <App />
            </ClerkLoaded>
          </ClerkProvider>
        </QueryClientProvider>
      </HelmetProvider>
    </BrowserRouter>
  </React.StrictMode>
);
