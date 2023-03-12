async function handleRequest(request) {
  let client = kv.client("ns")
  client.set("now", new Date())
  return new Response(client.get("now"));
}

addEventListener("fetch", async function (event) {
  event.respondWith(handleRequest(event.request));
});
