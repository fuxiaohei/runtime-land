import { createClient, format_axios_error } from "./client";

async function createOauthToken(req) {
    let client = createClient();
    try {
        let response = await client.post("/v1/token/oauth", req);
        console.log("createOauthToken:", req.name);
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function verifyToken(token) {
    let client = createClient();
    try {
        let response = await client.post("/v1/token/verify/" + encodeURIComponent(token));
        console.log("verifyToken:", token);
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function createDeploymentToken(name) {
    let client = createClient();
    try {
        let response = await client.post("/v1/token/deployment", { name: name });
        console.log("createDeploymentToken:", name);
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function listDeploymentTokens() {
    let client = createClient();
    try {
        let response = await client.get("/v1/token/deployment");
        console.log("listDeploymentTokens");
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function removeToken(uuid) {
    let client = createClient();
    try {
        let response = await client.delete("/v1/token/deployment/" + uuid);
        console.log("removeToken:", uuid);
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

export {
    createOauthToken,
    verifyToken,
    createDeploymentToken,
    listDeploymentTokens,
    removeToken,
};
