import { createClient, format_axios_error } from "./client";

async function list_projects() {
    let client = createClient();
    try {
        let response = await client.get("/v1/projects");
        console.log("list_projects");
        return response.data || {};
    } catch (error) {
        throw new Error(format_axios_error(error));
    }
}

async function remove_project(uuid) {
    let client = createClient();
    try {
        let response = await client.delete("/v1/project/" + uuid);
        console.log("remove_project:", uuid);
        return response.data || {};
    } catch (error) {
        throw new Error(format_axios_error(error));
    }
}

async function create_project(req) {
    let client = createClient();
    try {
        let response = await client.post("/v1/project", req);
        console.log("create_project:", req);
        return response.data || {};
    } catch (error) {
        throw new Error(format_axios_error(error));
    }
}


export {
    list_projects,
    remove_project,
    create_project,
}
