import { createClient, format_axios_error } from "./cloud";

async function listProjects() {
  let client = createClient();
  try {
    let response = await client.get("/v1/projects");
    let data = { data: response.data || [] };
    console.log("listProjects:", data.data.length);
    return data;
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

async function createProject(projectName, projectLanguage) {
  let client = createClient();
  try {
    let response = await client.post("/v1/project", {
      name: projectName,
      language: projectLanguage,
    });
    console.log("createProject:", projectName, projectLanguage);
    return response;
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

async function getProjectOverview(projectName, with_deployments) {
  let client = createClient();
  try {
    let response = await client.get("/v1/project/overview", {
      params: {
        name: projectName,
        language: "",
        with_deployments: with_deployments,
      }
    });
    console.log("getProjectOverview:", projectName, "with_deployments:", with_deployments);
    return response.data || {};
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

async function publishDeployment(deploy_id, deploy_uuid) {
  let client = createClient();
  try {
    let response = await client.post("/v1/deployment/publish", {
      deploy_id: deploy_id,
      deploy_uuid: deploy_uuid,
    });
    return response;
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

export { listProjects, createProject, publishDeployment, getProjectOverview };
