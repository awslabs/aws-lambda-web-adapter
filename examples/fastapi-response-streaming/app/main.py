import boto3
import json
import os
import uvicorn
from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.responses import RedirectResponse, StreamingResponse
from pydantic import BaseModel
from typing import Optional


app = FastAPI()

app.mount("/demo", StaticFiles(directory="static", html=True))


@app.get("/")
async def root():
    return RedirectResponse(url='/demo/')


class Story(BaseModel):
    topic: Optional[str] = None


@app.post("/api/story")
def api_story(story: Story):
    if story.topic == None or story.topic == "":
        return None

    return StreamingResponse(bedrock_stream(story.topic), media_type="text/html")


bedrock = boto3.client('bedrock-runtime')


async def bedrock_stream(topic: str):
    instruction = f"""
    You are a world class writer. Please write a sweet bedtime story about {topic}.
    """
    body = json.dumps({
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 1024,
        "messages": [
            {
                "role": "user",
                "content": instruction,
            }
        ],
    })

    response = bedrock.invoke_model_with_response_stream(
        modelId='anthropic.claude-3-haiku-20240307-v1:0',
        body=body
    )

    stream = response.get('body')
    if stream:
        for event in stream:
            chunk = event.get('chunk')
            if chunk:
                message = json.loads(chunk.get("bytes").decode())
                if message['type'] == "content_block_delta":
                    yield message['delta']['text'] or ""
                elif message['type'] == "message_stop":
                    yield "\n"


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=int(os.environ.get("PORT", "8080")))
