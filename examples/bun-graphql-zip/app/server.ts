import { createServer } from 'node:http'
import { createYoga } from 'graphql-yoga'
import { schema } from './schema.js'

// Create a Yoga instance with a GraphQL schema.
const yoga = createYoga({ schema })

// Pass it into a server to hook into request handlers.
const server = Bun.serve({
  fetch: yoga
})

// Start the server and you're done!
console.info(
  `Server is running on ${new URL(
    yoga.graphqlEndpoint,
    `http://${server.hostname}:${server.port}`
  )}`
)
