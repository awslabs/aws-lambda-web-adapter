const express = require('express')
const app = express()
const port = process.env['PORT'] || 8080


app.get('/Prod', (req, res) => {
    res.send('Hi there!')
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})