import { clientPost } from "./client";

async function signup(data) {
    return await clientPost("/v1/signup", data);
}

async function login_by_email(data) {
    return await clientPost("/v1/login", data);
}

export {
    login_by_email, signup
};
