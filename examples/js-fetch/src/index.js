export default {
    async fetch(request) {
        /**
        * Example someHost at URL is set up to respond with HTML
        * Replace URL with the host you wish to send requests to
        */
        const someHost = "http://ip-api.com";
        const url = someHost + "/json/1.1.1.1";

        /**
        * gatherResponse awaits and returns a response body as a string.
        * Use await gatherResponse(..) in an async function to get the response body
        * @param {Response} response
        */
        async function gatherResponse(response) {
            const { headers } = response;
            const contentType = headers.get("content-type") || "";
            if (contentType.includes("application/json")) {
                return JSON.stringify(await response.json());
            } else if (contentType.includes("application/text")) {
                return response.text();
            } else if (contentType.includes("text/html")) {
                return response.text();
            } else {
                return response.text();
            }
        }

        const init = {
            headers: {
                "content-type": "application/json;charset=UTF-8",
            },
        };
        const response = await fetch(url, init);
        const results = await gatherResponse(response);
        return new Response(results, init);
    },
};