import { createSchema } from 'graphql-yoga'

const typeDefs = /* GraphQL */ `
  type Query {
    hello(addition: String): String!
    info: String!
    feed: [Link!]!
  }
  type Mutation {
    postLink(url: String!, description: String!): Link!
  }
  type Link {
    id: ID!
    description: String!
    url: String!
  }
`
type Link = {
  id: string
  url: string
  description: string
}

const links: Link[] = [
  {
    id: 'link-0',
    url: 'www.howtographql.com',
    description: 'Fullstack tutorial for GraphQL'
  }
]

const resolvers = {
  Query: {
    hello: async (_: any, { addition }: { addition: string }) => {
      console.log(`Received addition: ${addition}`);
      return `Hello, ${addition || 'Lambda!'}!`
    },
    info: () => `This is the API of a Hackernews Clone`,
    feed: () => links
  },
  Mutation: {
    postLink: (parent: unknown, args: { description: string; url: string }) => {
      // 1
      let idCount = links.length

      // 2
      const link: Link = {
        id: `link-${idCount}`,
        description: args.description,
        url: args.url
      }

      links.push(link)

      return link
    }
  },
  Link: {
    id: (parent: Link) => parent.id,
    description: (parent: Link) => parent.description,
    url: (parent: Link) => parent.url
  }
}

export const schema = createSchema({
  typeDefs,
  resolvers
})

