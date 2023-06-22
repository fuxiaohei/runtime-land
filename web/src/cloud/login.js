import { createClient, format_axios_error } from "./cloud";

async function loginByEmail(email, password) {
  let client = createClient();
  try {
    let response = await client.post("/v1/login-by-email", {
      email: email,
      password: password,
    });
    console.log("loginByEmail:", email);
    return response.data || {};
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

async function loginByLocal(user) {
  let client = createClient();
  try {
    let response = await client.post("/v1/login-by-token", {
      access_token: user.access_token,
    });
    console.log("loginByLocal:", user.access_token);
    return response.data || {};
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

async function signupEmail(email, password, nickname) {
  let client = createClient();
  try {
    let response = await client.post("/v1/signup-email", {
      email: email,
      password: password,
      nickname: nickname,
    });
    console.log("signup:", email);
    return response.data || {};
  } catch (error) {
    return { error: format_axios_error(error) };
  }
}

export { loginByEmail, loginByLocal, signupEmail };
