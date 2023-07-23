const SERVER_URL = 'http://localhost:7777';

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
    let user = getLocalInfo();
    let headers = {};
    if (user && user.token) {
        headers["Authorization"] = "Bearer " + user.token.value;
    }
    const instance = axios.create({
        baseURL: SERVER_URL,
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


export {
    createClient,
    getLocalInfo,
    setLocalInfo,
    format_axios_error,
}