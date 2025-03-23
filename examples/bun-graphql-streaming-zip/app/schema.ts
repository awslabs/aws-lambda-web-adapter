import { createSchema } from 'graphql-yoga'

const typeDefs = /* GraphQL */ `
  type Query {
    hello(addition: String): String!
  }
  type Subscription {
    stream(addition: String): String!
  }
`
const resolvers = {
  Query: {
    hello: async (_: any, { addition }: { addition: string }) => {
      console.log(`Received addition: ${addition}`);
      return `Hello, ${addition || 'Lambda!'}!`
    },
  },
  Subscription: {
    stream: {
      subscribe: async function* (_: any, { addition }: { addition: string }) {
        const message = `This is streaming from Lambda! ${addition || ''}\n`;
        process.stdout.write("Streaming: ");
        for (const char of message) {
          process.stdout.write(char);
          yield char;
          // Sleep for 100ms between characters
          await new Promise(resolve => setTimeout(resolve, 100));
        }
      },
      resolve: (payload: any) => payload
    },
  }
}

export const schema = createSchema({
  typeDefs,
  resolvers
})

