import asyncio
from fastmcp import Client

client = Client("https://qeejoj5nk4.execute-api.ap-southeast-2.amazonaws.com/mcp")

async def call_tool(name: str):
    async with client:
        result = await client.call_tool("greet", {"name": name})
        print(result)

asyncio.run(call_tool("Ford"))