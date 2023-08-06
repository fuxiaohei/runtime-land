import { clientDelete, clientGet, clientPost } from "./client";

async function createOauthToken(req) {
    return await clientPost("/v1/token/oauth", req);
}

async function verifyToken(token) {
    return await clientPost("/v1/token/verify/" + encodeURIComponent(token));
}

async function createDeploymentToken(name) {
    return await clientPost("/v1/token/deployment", { name: name });
}

async function listDeploymentTokens() {
    return await clientGet("/v1/token/deployment");
}

async function removeToken(uuid) {
    return await clientDelete("/v1/token/deployment/" + uuid);
}

export {
    createOauthToken,
    verifyToken,
    createDeploymentToken,
    listDeploymentTokens,
    removeToken,
};
