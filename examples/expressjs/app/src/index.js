const express = require('express')
const app = express()
const port = process.env['PORT'] || 8080

// SIGTERM Handler
process.on('SIGTERM', async () => {
    console.info('[express] SIGTERM received');

    console.info('[express] cleaning up');
    // perform actual clean up work here.
    await new Promise(resolve => setTimeout(resolve, 100));

    console.info('[express] exiting');
    process.exit(0)
});

app.get('/', (req, res) => {
    res.send('Hi there!')
})

app.listen(port, () => {
    console.log(`Example app listening at http://localhost:${port}`)
})