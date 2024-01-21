const express = require('express')
const bodyParser = require('body-parser')

const app = express()
const port = process.env['PORT'] || 8080

// setup bodyParser to parse application/json
app.use(bodyParser.json())


// LWA sends SQS messages to this endpoint, use environment variable 'AWS_LWA_PASS_THROUGH_PATH' to configure this path
app.post('/events', (req, res) => {
    console.log(`Received event: ${JSON.stringify(req.body)}`)

    // printout the message Id and body
    req.body.Records.forEach((record) => {
        console.log(`processing message: ${record.messageId} with body: ${record.body}`)
    })

    // send a 200 response with json string 'success'
    res.json('success')
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})
