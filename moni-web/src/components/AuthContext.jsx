import React from "react";
import {
  getLocalUser,
  loginByLocalUser,
  loginByMail,
  signupByEmail,
} from "../api/login";
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
    if (newUser.email && newUser.password) {
      let res = await loginByMail(newUser.email, newUser.password);
      if (!res.error) {
        setUser(res);
        localStorage.setItem("moni-web-user", JSON.stringify(res));
      }
      return res;
    }
  };

  let signup = async (newUser) => {
    if (newUser.email && newUser.password) {
      let res = await signupByEmail(
        newUser.email,
        newUser.password,
        newUser.nickname
      );
      if (!res.error) {
        setUser(res);
        localStorage.setItem("moni-web-user", JSON.stringify(res));
      }
      return res;
    }
  };

  let signout = async (user) => {
    localStorage.removeItem("moni-web-user");
    setUser(null);
  };

  let value = { user, signin, signout, signup };
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
    return <Navigate to="/login-email" state={{ from: location }} replace />;
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
    return <Navigate to="/projects" state={{ from: location }} replace />;
  }

  return children;
}

function SignoutPage() {
  let auth = userAuthContext();
  const navigate = useNavigate();
  const handleLogout = async () => {
    await auth.signout(auth.user);
    navigate("/login-email");
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
