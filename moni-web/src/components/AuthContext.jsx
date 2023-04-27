import React from "react";
import { mockAuthProvider } from "../auth";
import { getLocalUser, loginByEmail } from "../api/login";
import { Navigate, useLocation } from "react-router-dom";

let AuthContext = React.createContext(null);

function userAuthContext() {
  return React.useContext(AuthContext);
}

function AuthProvider({ children }) {
  let localUser = getLocalUser();
  let [user, setUser] = React.useState(localUser);

  let signin = async (newUser) => {
    // if login by email and password
    if (newUser.email && newUser.password) {
      let res = await loginByEmail(newUser.email, newUser.password);
      console.log("---res", res);
      if (res.data) {
        setUser(res.data);
        localStorage.setItem("moni-web-user", JSON.stringify(res.data));
      }
      return res;
    }
  };

  let signout = (callback) => {
    return mockAuthProvider.signout(() => {
      setUser(null);
      callback();
    });
  };

  let value = { user, signin, signout };
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

function RequireAuth({ children }) {
  let auth = userAuthContext();
  console.log("---auth", auth, children);
  let location = useLocation();

  if (!auth.user) {
    // Redirect them to the /login page, but save the current location they were
    // trying to go to when they were redirected. This allows us to send them
    // along to that page after they login, which is a nicer user experience
    // than dropping them off on the home page.
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  return children;
}

function RequireUnauth({ children }) {
  let auth = userAuthContext();
  let location = useLocation();

  if (auth.user) {
    console.log("RequireUnauth", auth.user, location);
    return <Navigate to="/dashboard" state={{ from: location }} replace />;
  }

  return children;
}

export {
  userAuthContext,
  AuthContext,
  AuthProvider,
  RequireAuth,
  RequireUnauth,
};
