import { callClient } from "./utils.js";

const { Empty, ProjectOverviewRequest } = require("./proto/moni-rpc_pb.js");

async function listProjects() {
  let req = new Empty();
  let response = await callClient(req, "listProjects");
  return response;
}

async function getProjectOverview(projectName) {
  let req = new ProjectOverviewRequest();
  req.setName(projectName);
  let response = await callClient(req, "projectOverview");
  return response;
}

export { listProjects, getProjectOverview };
