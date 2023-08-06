import { clientGet } from "./client";

// listRegions returns a list of regions.
async function listRegions() {
    return await clientGet("/v1/regions");
}

export {
    listRegions,
}