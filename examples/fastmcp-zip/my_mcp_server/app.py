from fastmcp import FastMCP
from starlette.responses import JSONResponse

mcp = FastMCP("My MCP Server")


@mcp.tool
def greet(name: str) -> str:
    return f"Hello, {name}!"


@mcp.custom_route("/healthz", methods=["GET"])
async def health_check(request):
    return JSONResponse({"status": "ok"})


app = mcp.http_app(transport="http", stateless_http=True, json_response=True)

