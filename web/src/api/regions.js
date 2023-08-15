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

// listStorageSettings returns storage settings.
async function listStorageSettings() {
    return await clientGet("/v1/settings/storage")
}

// updateStorageSettings updates storage setting.
async function updateStorageSettings({ typename, storage }) {
    return await clientPost("/v1/settings/storage?typename=" + typename, storage)
}

// getStats returns stats.
async function getStats() {
    return clientGet("/v1/settings/stats")
}

export {
    listRegions,
    listDomainSettings,
    updateDomainSettings,
    listStorageSettings,
    updateStorageSettings,
    getStats,
}