import React, { useEffect } from "react";
import {
  SignedIn,
  SignedOut,
  RedirectToSignIn,
  useUser,
} from "@clerk/clerk-react";
import { getLocalInfo, setLocalInfo } from "../api/client";
import { createToken } from "../api/token";

const AuthContext = React.createContext(null);

function useAuthContext() {
  return React.useContext(AuthContext);
}

function AuthProvider({ children }) {
  const { isLoaded, isSignedIn, user } = useUser();
  const [getTokenSuccess, setGetTokenSuccess] = React.useState(false);
  const [getTokenError, setGetTokenError] = React.useState(null);

  const fetchToken = async (req) => {
    let response = await createToken(req);
    if (response.error) {
      setGetTokenError(response.error);
      setGetTokenSuccess(false);
      return;
    }
    setGetTokenSuccess(true);
    let value = {
      user: {
        name: response.nick_name,
        email: response.email,
        avatar_url: response.avatar_url,
        oauth_id: response.oauth_id,
      },
      token: {
        value: response.token_value,
        uuid: response.token_uuid,
        expired_at: response.token_expired_at,
      },
    };
    setLocalInfo(value);
  };

  useEffect(() => {
    if (!isLoaded || !isSignedIn) {
      return;
    }
    let localInfo = getLocalInfo();
    if (localInfo && localInfo.token) {
      const current_timestamp = new Date().getTime();
      if (
        current_timestamp / 1000 < localInfo.token.expired_at &&
        localInfo.user.oauth_id == user.id
      ) {
        console.log("local token is valid");
        setGetTokenSuccess(true);
        return;
      }
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
    fetchToken(req);
  }, [isSignedIn]);

  if (!isLoaded || !isSignedIn) {
    return (
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    );
  }

  if (!getTokenSuccess) {
    return <h1>Error...</h1>;
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
