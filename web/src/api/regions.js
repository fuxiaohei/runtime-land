import { clientGet, clientPost } from "./client";

// listRegions returns a list of regions.
async function listRegions() {
    return await clientGet("/v1/regions");
}

// listDomainSettings returns domain settings.
async function listDomainSettings() {
    return await clientGet("/v1/settings/domains")
}

// updateDomainSettings updates domain setting.
async function updateDomainSettings({ domain, protocol }) {
    return await clientPost("/v1/settings/domains", { domain, protocol })
}

export {
    listRegions,
    listDomainSettings,
    updateDomainSettings,
}