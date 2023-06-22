const RPC_CLIENT_ADDRESS = window.API_ADDRESS || "http://127.0.0.1:38779";

const axios = require("axios");

function getLocalUser() {
  let local_user = localStorage.getItem("runtime-land-user") || null;
  if (local_user) {
    local_user = JSON.parse(local_user);
  }
  return local_user;
}

function removeLocalUser() {
  localStorage.removeItem("runtime-land-user");
}

function setLocalUser(user) {
  console.log("setLocalUser:", user);
  user.lastVerifyTime = Date.now();
  localStorage.setItem("runtime-land-user", JSON.stringify(user));
}

function createClient() {
  let user = getLocalUser();
  let headers = {};
  if (user && user.access_token) {
    headers["Authorization"] = "Bearer " + user.access_token;
  }
  const instance = axios.create({
    baseURL: RPC_CLIENT_ADDRESS,
    timeout: 20000,
    headers: headers,
  });
  return instance;
}

function format_axios_error(error) {
  let message = error.toString();
  if (error.response) {
    let data = error.response.data;
    if (typeof data === "object") {
      if (data.message) {
        message = data.message;
      } else {
        message = JSON.stringify(data);
      }
    } else {
      message = data;
    }
  }
  return message;
}

export {
  RPC_CLIENT_ADDRESS,
  createClient,
  getLocalUser,
  removeLocalUser,
  setLocalUser,
  format_axios_error
};
