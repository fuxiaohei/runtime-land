console.log("API_URL", API_URL);

const axios = require("axios");

function getLocalInfo() {
    const info = localStorage.getItem("runtime-land-user-info");
    if (info) {
        return JSON.parse(info);
    }
    return null;
}

function setLocalInfo(info) {
    localStorage.setItem("runtime-land-user-info", JSON.stringify(info));
}

function createClient() {
    let info = getLocalInfo();
    let headers = {};
    if (info && info.token) {
        headers["Authorization"] = "Bearer " + info.token.value;
    }
    const instance = axios.create({
        baseURL: API_URL,
        timeout: 20000,
        headers: headers,
    });
    return instance;
}

function format_axios_error(error) {
    let message = error.toString();
    if (error.response) {
        let data = error.response.data;
        if (typeof data === "object") {
            if (data.message) {
                message = data.message;
            } else {
                message = JSON.stringify(data);
            }
        } else {
            message = data;
        }
    }
    return message;
}

async function clientGet(url) {
    let client = createClient();
    try {
        let response = await client.get(url);
        return response.data || {};
    } catch (error) {
        throw format_axios_error(error);
    }
}

async function clientDelete(url) {
    let client = createClient();
    try {
        let response = await client.delete(url);
        return response.data || {};
    } catch (error) {
        throw format_axios_error(error);
    }
}

async function clientPost(url, req) {
    let client = createClient();
    try {
        let response = await client.post(url, req);
        return response.data || {};
    } catch (error) {
        throw format_axios_error(error);
    }
}

export {
    clientGet,
    clientDelete,
    clientPost,
    createClient,
    getLocalInfo,
    setLocalInfo,
    format_axios_error,
}