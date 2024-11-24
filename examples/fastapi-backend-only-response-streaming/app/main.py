import boto3
import json
import os
import uvicorn
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
import asyncio


BEDROCK_MODEL = os.environ.get(
    "BEDROCK_MODEL", "anthropic.claude-3-haiku-20240307-v1:0"
)
SYSTEM = os.environ.get("SYSTEM", "You are a helpful assistant.")

app = FastAPI()
bedrock = boto3.Session().client("bedrock-runtime")


# Define the request model
class QueryRequest(BaseModel):
    query: str


@app.get("/api/stream")
async def api_stream(request: QueryRequest):
    if not request.query:
        return None

    return StreamingResponse(
        bedrock_stream(request.query),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
        },
    )


async def bedrock_stream(query: str):
    instruction = f"""
    You are a helpful assistant. Please provide an answer to the user's query
    <query>{query}</query>.
    """
    body = json.dumps(
        {
            "anthropic_version": "bedrock-2023-05-31",
            "max_tokens": 1024,
            "system": SYSTEM,
            "temperature": 0.1,
            "top_k": 10,
            "messages": [
                {
                    "role": "user",
                    "content": instruction,
                }
            ],
        }
    )

    response = bedrock.invoke_model_with_response_stream(
        modelId=BEDROCK_MODEL, body=body
    )

    stream = response.get("body")
    if stream:
        for event in stream:
            chunk = event.get("chunk")
            if chunk:
                message = json.loads(chunk.get("bytes").decode())
                if message["type"] == "content_block_delta":
                    yield message["delta"]["text"] or ""
                    await asyncio.sleep(0.01)
                elif message["type"] == "message_stop":
                    yield "\n"
                    await asyncio.sleep(0.01)


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=int(os.environ.get("PORT", "8080")))
