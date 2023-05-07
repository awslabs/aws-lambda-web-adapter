from fastapi import FastAPI
from fastapi.responses import StreamingResponse
import asyncio

app = FastAPI()


async def streamer():
    for i in range(10):
        await asyncio.sleep(1)
        yield b"This is streaming from Lambda \n"


@app.get("/")
async def index():
    return StreamingResponse(streamer(), media_type="text/plain; charset=utf-8")
