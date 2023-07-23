import { createClient, format_axios_error } from "./client";

async function createToken(req) {
    let client = createClient();
    try {
        let response = await client.post("/v1/token", req);
        console.log("createToken:", req.name);
        return response.data || {};
    } catch (error) {
        return { error: format_axios_error(error) };
    }
}

export { createToken }