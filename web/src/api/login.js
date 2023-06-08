const {
  LoginEmailRequest,
  LoginAccessTokenRequest,
  SignupEmailRequest,
} = require("./proto/moni-rpc_pb.js");
const { MoniRpcServiceClient } = require("./proto/moni-rpc_grpc_web_pb.js");
const { RPC_CLIENT_ADDRESS, callClient } = require("./utils.js");

function loginByEmail(email, password) {
  let client = new MoniRpcServiceClient(RPC_CLIENT_ADDRESS);
  let request = new LoginEmailRequest();
  request.setEmail(email);
  request.setPassword(password);

  let promise = new Promise((resolve, reject) => {
    client.loginEmail(request, {}, (err, response) => {
      if (err) {
        resolve({ code: 1, error: err });
        return;
      }
      if (response.getCode()) {
        resolve({ code: response.getCode(), error: response.getError() });
        return;
      }
      let data = response.getData().toObject();
      resolve({ code: 0, data: data });
    });
  });
  return promise;
}

function getLocalUser() {
  let local_user = localStorage.getItem("moni-web-user") || null;
  if (local_user) {
    local_user = JSON.parse(local_user);
  }
  return local_user;
}

function setLocalUser(user) {
  console.log("setLocalUser:", user)
  user.lastVerifyTime = Date.now();
  localStorage.setItem("moni-web-user", JSON.stringify(user));
}

function removeLocalUser() {
  localStorage.removeItem("moni-web-user");
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
