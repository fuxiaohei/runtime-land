import { createClient, format_axios_error } from "./client";

async function list_regions() {
    let client = createClient();
    try {
        let response = await client.get("/v1/regions");
        console.log("list_regions");
        return response.data || {};
    } catch (error) {
        throw new Error(format_axios_error(error));
    }
}

export {
    list_regions,
}