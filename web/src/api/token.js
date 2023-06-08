import { callClient } from "./utils.js";

const {
  CreateAccessTokenRequest,
  RemoveAccessTokenRequest,
  Empty,
} = require("./proto/moni-rpc_pb.js");

async function createAccessToken(name) {
  let req = new CreateAccessTokenRequest();
  req.setName(name);
  let response = await callClient(req, "createAccessToken");
  return response;
}

async function listAccessTokens() {
  let req = new Empty();
  let response = await callClient(req, "listAccessTokens");
  return response;
}

async function removeAccessToken(uuid) {
  let req = new RemoveAccessTokenRequest();
  req.setTokenUuid(uuid);
  let response = await callClient(req, "removeAccessToken");
  return response;
}

export { createAccessToken, listAccessTokens, removeAccessToken };
