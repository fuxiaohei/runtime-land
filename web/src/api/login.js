const {
  LoginEmailRequest,
  LoginAccessTokenRequest,
  SignupEmailRequest,
} = require("./proto/land-rpc_pb.js");
const { callClient } = require("./utils.js");

function getLocalUser() {
  let local_user = localStorage.getItem("runtime-land-user") || null;
  if (local_user) {
    local_user = JSON.parse(local_user);
  }
  return local_user;
}

function setLocalUser(user) {
  console.log("setLocalUser:", user);
  user.lastVerifyTime = Date.now();
  localStorage.setItem("runtime-land-user", JSON.stringify(user));
}

function removeLocalUser() {
  localStorage.removeItem("runtime-land-user");
}

async function loginByLocalUser(user) {
  let req = new LoginAccessTokenRequest();
  req.setAccessToken(user.accessToken);
  let response = await callClient(req, "loginAccessToken");
  return response;
}

async function loginByMail(email, password) {
  let request = new LoginEmailRequest();
  request.setEmail(email);
  request.setPassword(password);
  let response = await callClient(request, "loginEmail");
  return response;
}

async function signupByEmail(email, password, nickname) {
  let request = new SignupEmailRequest();
  request.setEmail(email);
  request.setPassword(password);
  request.setNickname(nickname);
  let response = await callClient(request, "signupEmail");
  return response;
}

export {
  loginByEmail,
  getLocalUser,
  setLocalUser,
  removeLocalUser,
  loginByLocalUser,
  loginByMail,
  signupByEmail,
};
