import { callClient } from "./utils.js";

const { Empty } = require("./proto/moni-rpc_pb.js");

async function listProjects() {
  let req = new Empty();
  let response = await callClient(req, "listProjects");
  return response;
}

export { listProjects };
