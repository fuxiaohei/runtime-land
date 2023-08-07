import { clientGet } from "./client";

// listRegions returns a list of regions.
async function listRegions() {
    return await clientGet("/v1/regions");
}

// listDomainSettings returns a list of domain settings.
async function listDomainSettings(){
    return await clientGet("/v1/settings/domains")
}

export {
    listRegions,
    listDomainSettings,
}