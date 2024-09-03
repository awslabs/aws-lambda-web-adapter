from fasthtml.common import *
from starlette.responses import StreamingResponse
import asyncio
import secrets

app, rt = fast_app(
        ct_hdr=True,
        hdrs=[Script(src="https://unpkg.com/htmx-ext-transfer-encoding-chunked@0.4.0/transfer-encoding-chunked.js")],
        secret_key=secrets.token_hex(32),
        debug=True,
)

async def streamer():
    message = "This is streaming from AWS Lambda! 🚀\n"
    response_txt = ""
    for char in message:
        response_txt += char
        yield to_xml(Div(
                        response_txt,
                        id=f"stream-output",
                        hx_swap_oob="outerHTML",
                    ))
        await asyncio.sleep(0.1)

@rt("/")
def index():
    return Div(
        Button("Click to stream", hx_get=stream, hx_target="#stream-output", hx_ext="chunked-transfer"), 
        P(id="stream-output")
    )

@app.get
async def stream():
    return StreamingResponse(streamer(), media_type="text/plain", 
                             headers={"X-Transfer-Encoding": "chunked"})

serve(port=8000, reload=True)