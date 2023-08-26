const express = require('express')
const app = express()
const port = process.env['PORT'] || 8080


app.get('/', (req, res) => {
    // deserialize request context from the http header 'x-amzn-request-context'
    let context = req.headers['x-amzn-request-context'] || null
    let requestContext = context != null? JSON.parse(context) : null

    let lambda = req.headers['x-amzn-lambda-context'] || null
    let lambdaContext = lambda != null? JSON.parse(lambda) : null
    res.send({
        messagge: 'Hi there!', 
        requestContext,
        lambdaContext
    })
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})