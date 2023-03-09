async function handleRequest(request) {
  let response = await fetch("https://www.rust-lang.org/");
  return response;
}

addEventListener("fetch", async function (event) {
  event.respondWith(handleRequest(event.request));
});
