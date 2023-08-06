import { clientDelete, clientPost, clientGet } from "./client";

async function listProjects() {
    return await clientGet("/v1/projects");
}

async function getProjectOverview(name) {
    return await clientGet("/v1/project/" + name + "/overview");
}

async function getProject(name) {
    return await clientGet("/v1/project/" + name);
}

async function removeProject(uuid) {
    return await clientDelete("/v1/project/" + uuid);
}

async function createProject(req) {
    return await clientPost("/v1/project", req);
}

export {
    listProjects,
    removeProject,
    createProject,
    getProjectOverview,
    getProject,
}
