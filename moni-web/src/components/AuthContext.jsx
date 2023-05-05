import React from "react";
import { getLocalUser, loginByEmail, loginByLocalUser } from "../api/login";
import { Navigate, useNavigate, useLocation } from "react-router-dom";
import LoginLoadingPage from "../pages/LoginLoadingPage";

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
      if (res.data) {
        setUser(res.data);
        localStorage.setItem("moni-web-user", JSON.stringify(res.data));
      }
      return res;
    }
  };

  let signout = async (user) => {
    localStorage.removeItem("moni-web-user");
    setUser(null);
  };

  let value = { user, signin, signout };
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

function RequireAuth({ children }) {
  let auth = userAuthContext();
  let location = useLocation();
  let [logged, setLogged] = React.useState(false);

  const fetchLogin = async () => {
    let response = await loginByLocalUser(auth.user);
    if (response.error) {
      console.log("[auth] browser token error", response.error);
      auth.signout();
      return;
    }
    setLogged(true);
  };

  React.useEffect(() => {
    if (!auth.user) {
      return;
    }
    fetchLogin();
  });

  if (!auth.user) {
    console.log("[auth] no browser token");
    // Redirect them to the /login page, but save the current location they were
    // trying to go to when they were redirected. This allows us to send them
    // along to that page after they login, which is a nicer user experience
    // than dropping them off on the home page.
    return <Navigate to="/login" state={{ from: location }} replace />;
  }
  if (!logged) {
    return <LoginLoadingPage />;
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

function SignoutPage() {
  let auth = userAuthContext();
  const navigate = useNavigate();
  const handleLogout = async () => {
    await auth.signout(auth.user);
    navigate("/login");
  };

  // Call handleLogout when the component is mounted
  React.useEffect(() => {
    handleLogout();
  }, []);

  return (
    <div id="logging-out">
      <h4>Logging out...</h4>
    </div>
  );
}

export {
  userAuthContext,
  AuthContext,
  AuthProvider,
  RequireAuth,
  RequireUnauth,
  SignoutPage,
};
