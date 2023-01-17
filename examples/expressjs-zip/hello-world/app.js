const express = require('express')
const app = express()
const port = process.env['PORT'] || 8080


app.get('/', (req, res) => {
    // deserialize request context from the http header 'x-amzn-request-context'
    let requestContext = JSON.parse(req.headers['x-amzn-request-context'])
    res.send({
        messagge: 'Hi there!', 
        requestContext
    })
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})