// habitica.ts
var opts = {
  "habitica.api_url": {
    env_var_name: "PPC_HABITICA_API_URL",
    default_val: "https://habitica.com/api/v3"
  },
  "habitica.user_id": {
    env_var_name: "PPC_HABITICA_USER_ID"
  },
  "habitica.client_name": {
    default_val: "3544a0b8-d71a-46e0-9bb1-6ddbb2abcddb-PPC-Software"
  },
  "habitica.api_token": {
    env_var_name: "PPC_HABITICA_API_TOKEN"
  }
};
async function habitica(mize) {
  console.log("hiiiiiiiiiiiiiiiiiiii");
  mize.add_opts(opts);
  mize.add_part(new Habitica(mize));
}
async function handleRateLimit(response) {
  const limit = response.headers.get("X-RateLimit-Limit") || "NONE";
  const remaining = parseInt(response.headers.get("X-RateLimit-Remaining") || "10", 10);
  const reset = response.headers.get("X-RateLimit-Reset");
  console.log(`RateLimit: ${limit} | Remaining: ${remaining} | Reset: ${reset}`);
  if (remaining < 2 && reset) {
    const resetDate = new Date(reset);
    const now = /* @__PURE__ */ new Date();
    const waitMs = resetDate.getTime() - now.getTime() + 1e3;
    if (waitMs > 0) {
      console.log(`Waiting ${Math.round(waitMs / 1e3)} secs for next rate limit window...`);
      await new Promise((resolve) => setTimeout(resolve, waitMs));
    }
  }
}
var Habitica = class {
  mize;
  constructor(mize) {
    this.mize = mize;
  }
  async get_tasks(type = "todos") {
    return await this.api_request("GET", `tasks/user?type=${type}`);
  }
  async delete_task(id) {
    await this.api_request("DELETE", `tasks/${id}`);
  }
  async api_request(method, path, extraHeaders = {}, data = {}) {
    const mize = this.mize;
    if (!path.startsWith("/")) {
      path = "/" + path;
    }
    const headers = Object.assign({
      "Content-Type": "application/json",
      "x-api-user": mize.get_config("habitica.user_id"),
      "x-api-key": mize.get_config("habitica.api_token"),
      "x-client": mize.get_config("habitica.client_name")
    }, extraHeaders);
    const response = await fetch(mize.get_config("habitica.api_url") + path, {
      method,
      headers,
      body: JSON.stringify(data)
    });
    await handleRateLimit(response);
    if (!response.ok) {
      console.log("request_url:", mize.get_config("habitica.api_url") + path);
      throw new Error(`Habitica API error: ${response.status} ${response.statusText}`);
    }
    return (await response.json()).data;
  }
};
export {
  habitica,
  opts
};
