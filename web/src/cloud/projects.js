import { createClient } from "./cloud";

async function listProjects() {
  let client = createClient();
  try {
    let response = await client.get("/v1/projects");
    let data = { data: response.data || [] };
    console.log("listProjects:", data.data.length);
    return data;
  } catch (error) {
    return { error: error };
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
    return { error: error };
  }
}

export { listProjects, createProject };
