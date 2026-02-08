from fastapi import FastAPI
from fastapi.responses import StreamingResponse
import asyncio
import uuid

app = FastAPI()

@app.get("/health")
async def health():
    return {"status": "healthy"}


async def streamer(request_id: str):
    """Stream a message character by character with request ID for tracing."""
    message = f"[{request_id}] This is streaming from Lambda Managed Instances!\n"
    for char in message:
        yield char
        await asyncio.sleep(0.05)


@app.get("/")
async def index():
    """Stream response - each concurrent request gets a unique ID."""
    request_id = str(uuid.uuid4())[:8]
    return StreamingResponse(streamer(request_id), media_type="text/plain")
