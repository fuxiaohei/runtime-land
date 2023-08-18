import {
  RedirectToSignIn,
  SignedIn,
  SignedOut,
  useClerk,
  useUser,
} from "@clerk/clerk-react";
import { useQuery } from "@tanstack/react-query";
import React from "react";
import {
  getLocalInfo,
  handleTokenResponse,
  removeLocalInfo,
} from "../api/client";
import { createOauthToken, verifyToken } from "../api/token";
import { selfHost } from "../config";
import ErrorPage from "../pages/Error";
import LoadingPage from "../pages/Loading";

const AuthContext = React.createContext(null);

function useAuthContext() {
  return React.useContext(AuthContext);
}

async function verifyLocalToken() {
  let localInfo = getLocalInfo();
  if (localInfo && localInfo.token) {
    let now_ts = Date.now() / 1000;
    // if token is in active interval, use local token
    let active_at = localInfo.token.active_at;
    if (now_ts - active_at < localInfo.token.active_interval) {
      console.log("local token is in active interval");
      return true;
    }
    // if token is not expired, use local token
    let expired_at = localInfo.token.expired_at;
    if (expired_at && expired_at > now_ts) {
      console.log("local token is valid");
      let response = await verifyToken(localInfo.token.value);
      handleTokenResponse(response);
      console.log("verify local token");
      return true;
    }
  }
  return false;
}

function AuthProvider({ children }) {
  return selfHost
    ? SelfHostAuthProvider({ children })
    : ClerkAuthProvider({ children });
}

function SelfHostAuthProvider({ children }) {
  const { isLoading, isError, error } = useQuery({
    queryKey: ["auth-context-selfhost"],
    queryFn: async () => {
      return await verifyLocalToken();
    },
    retry: false,
  });

  if (isLoading) {
    return <LoadingPage />;
  }

  if (isError) {
    return <ErrorPage message={error.toString()} />;
  }

  let v = getLocalInfo();
  v.signOut = async () => {
    removeLocalInfo();
    window.location.reload(); // it will redirect to login page
  };

  return <AuthContext.Provider value={v}>{children}</AuthContext.Provider>;
}

function ClerkAuthProvider({ children }) {
  const { isLoaded, isSignedIn, user } = useUser();
  const { signOut } = useClerk();

  const { isLoading, isError, error } = useQuery({
    queryKey: ["auth-context"],
    queryFn: async () => {
      if (await verifyLocalToken()) {
        return true;
      }
      console.log("local token is invalid, fetch new token");
      let req = {
        name: user.username || user.firstName,
        display_name: user.fullName,
        email: user.primaryEmailAddress.emailAddress,
        image_url: user.imageUrl,
        oauth_id: user.id,
        oauth_provider: "clerk",
        oauth_social: user.primaryEmailAddress.linkedTo[0].type,
      };
      let response = await createOauthToken(req);
      handleTokenResponse(response);
      window.location.reload();
      return true;
    },
    retry: false,
    enabled: isLoaded && isSignedIn,
  });

  if (!isLoaded || !isSignedIn) {
    return (
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    );
  }

  if (isLoading) {
    return <LoadingPage />;
  }

  if (isError) {
    return <ErrorPage message={error.toString()} />;
  }

  let v = getLocalInfo();
  v.signOut = async () => {
    signOut();
  };

  return (
    <AuthContext.Provider value={v}>
      <SignedIn>{children}</SignedIn>
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    </AuthContext.Provider>
  );
}

export { AuthProvider, useAuthContext };
