import { callClient } from "./utils.js";

const { CreateAccessTokenRequest, Empty } = require("./proto/moni-rpc_pb.js");

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

export { createAccessToken, listAccessTokens };
