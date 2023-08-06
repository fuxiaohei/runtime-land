import React from "react";
import {
  SignedIn,
  SignedOut,
  RedirectToSignIn,
  useUser,
} from "@clerk/clerk-react";
import { getLocalInfo, setLocalInfo } from "../api/client";
import { createOauthToken, verifyToken } from "../api/token";
import { useQuery } from "@tanstack/react-query";

const AuthContext = React.createContext(null);

function useAuthContext() {
  return React.useContext(AuthContext);
}

function AuthProvider({ children }) {
  const { isLoaded, isSignedIn, user } = useUser();

  const handleTokenResponse = (response) => {
    let value = {
      user: {
        name: response.nick_name,
        email: response.email,
        avatar_url: response.avatar_url,
        oauth_id: response.oauth_id,
        role: response.role,
      },
      token: {
        value: response.token_value,
        uuid: response.token_uuid,
        expired_at: response.token_expired_at,
      },
    };
    setLocalInfo(value);
  };

  const { isLoading, isError, error } = useQuery({
    queryKey: ["auth-context"],
    queryFn: async () => {
      let localInfo = getLocalInfo();
      if (localInfo && localInfo.token) {
        let response = await verifyToken(localInfo.token.value);
        handleTokenResponse(response);
        console.log("verify local token");
        return true;
      }
      console.log("local token is invalid, fetch new token", user);
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
    return <h1>Loading...</h1>;
  }

  if (isError) {
    return <h1>Error:{error.toString()}</h1>;
  }

  return (
    <AuthContext.Provider value={getLocalInfo()}>
      <SignedIn>{children}</SignedIn>
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    </AuthContext.Provider>
  );
}

export { AuthProvider, useAuthContext };
