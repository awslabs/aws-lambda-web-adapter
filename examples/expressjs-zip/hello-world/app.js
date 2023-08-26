const express = require('express')
const app = express()
const port = process.env['PORT'] || 8080


app.get('/', (req, res) => {
    // deserialize request context from the http header 'x-amzn-request-context'
    let context = req.headers['x-amzn-request-context'] || null
    let requestContext = context != null? JSON.parse(context) : null
    res.send({
        messagge: 'Hi there!', 
        requestContext
    })
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})