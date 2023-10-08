const {
    error,      // creates error responses
    text,       // creates text responses
    json,       // creates JSON responses
    Router,     // the ~440 byte router itself
    withParams, // middleware: puts params directly on the Request
} = itty;

// create a new Router
const router = Router()

router
    .get('/hello', (request) => text("Hello, World"))
    .get('/json', (request) => json({ hello: "world" }))
    .post('/foo/bar', async (request) => {
        const body = await request.text()
        return text("Body: " + body)
    })
    .get("/params/:value", withParams, ({ value }) => {
        return text(`value: ${value}`)
    })
    .all('*', () => error(404))

export default {
    fetch: (request) =>
        router.handle(request).catch(error),
}