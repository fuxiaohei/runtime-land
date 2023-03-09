async function handleRequest(request) {
  return new Response("Hello world!", {
    headers: {
      "content-type": "text/plain",
      "x-request-method": request.method,
      "x-request-url": request.url,
    },
  });
}

addEventListener("fetch", async function (event) {
  event.respondWith(handleRequest(event.request));
});
