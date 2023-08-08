import { clientPost } from "./client";


async function publishDeployment(uuid) {
    return await clientPost("/v1/deployment/" + uuid + "/publish");
}

async function disableDeployment(uuid) {
    return await clientPost("/v1/deployment/" + uuid + "/disable");
}

async function enableDeployment(uuid) {
    return await clientPost("/v1/deployment/" + uuid + "/enable");
}

export {
    publishDeployment,
    disableDeployment,
    enableDeployment,
}