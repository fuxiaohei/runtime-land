import { callClient } from "./utils.js";

const {
  Empty,
  ProjectOverviewRequest,
  FetchProjectRequest,
  PublishDeploymentRequest,
} = require("./proto/land-rpc_pb.js");

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

async function createEmptyProject(projectName, projectLanguage) {
  let req = new FetchProjectRequest();
  req.setName(projectName);
  req.setLanguage(projectLanguage);
  let response = await callClient(req, "createEmptyProject");
  return response;
}

async function publishDeployment(deployID, deployUUID) {
  let req = new PublishDeploymentRequest();
  req.setDeployId(deployID);
  req.setDeployUuid(deployUUID);
  let response = await callClient(req, "publishDeployment");
  return response;
}

export { listProjects, getProjectOverview, createEmptyProject, publishDeployment };
