import { useUser } from "@clerk/clerk-react";
import React from "react";
import {
  SignedIn,
  SignedOut,
  RedirectToSignIn,
  useUser,
} from "@clerk/clerk-react";

const AuthContext = React.createContext(null);

function userAuthContext() {
  return React.useContext(AuthContext);
}

function AuthProvider({ children }) {
  const value = useUser();
  console.log("---user",value)
  return (
    <AuthContext.Provider value={value}>
      <SignedIn>{children}</SignedIn>
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    </AuthContext.Provider>
  );
}

export { AuthProvider, userAuthContext };
