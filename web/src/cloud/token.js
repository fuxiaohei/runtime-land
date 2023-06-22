import { createClient, format_axios_error } from "./cloud";


async function createToken(name) {
    let client = createClient();
    try {
        let response = await client.post("/v1/tokens", {
            name: name,
        });
        console.log("createToken:", name);
        return response;
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function listTokens() {
    let client = createClient();
    try {
        let response = await client.get("/v1/tokens");
        let data = response.data || []
        console.log("listTokens:", data.length);
        return data;
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

async function removeToken(uuid) {
    let client = createClient();
    try {
        let response = await client.delete("/v1/tokens", {
            params: {
                uuid: uuid
            }
        });
        console.log("removeToken:", uuid);
        return response;
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

export { createToken, listTokens, removeToken };