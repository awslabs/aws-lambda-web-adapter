from fastapi import FastAPI
from fastapi.responses import StreamingResponse
import asyncio

app = FastAPI()


async def streamer():
    message = "This is streaming from Lambda!\n"
    for char in message:
        yield char
        await asyncio.sleep(0.1)


@app.get("/")
async def index():
    return StreamingResponse(streamer(), media_type="text/html")
