import React from "react";
import { selector, useRecoilValue } from "recoil";

const AuthContext = React.createContext(null);

const userInfo = selector({
  key: "userInfo",
  get: () => {
    console.log("get user info");
    return { logged: true, user: { name: "testing-user", token: "ttt" } };
  },
});

function userAuthContext() {
  return React.useContext(AuthContext);
}

function AuthProvider({ children }) {
  let user = useRecoilValue(userInfo);
  let value = { user };
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export { AuthProvider, userAuthContext };
