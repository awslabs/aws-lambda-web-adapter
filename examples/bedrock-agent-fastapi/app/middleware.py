import json
from urllib.parse import urlencode

from fastapi import Response
from fastapi.responses import JSONResponse
from starlette.middleware.base import BaseHTTPMiddleware


class BedrockAgentMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        # pass through any non-events requests
        if request.url.path != "/events":
            return await call_next(request)

        # convert the request body to json object
        req_body = await request.body()
        req_body = json.loads(req_body)

        request.scope["path"] = req_body["apiPath"]
        request.scope["method"] = req_body["httpMethod"]

        # query params
        params = {}
        parameters = req_body.get("parameters", [])
        for item in parameters:
            params[item["name"]] = item["value"]
        request.scope["query_string"] = urlencode(params).encode()

        # body
        content = req_body.get("requestBody", {}).get("content", None)
        if content:
            for key in content.keys():
                content_type = key
                break

            data = {}
            content_val = content.get(content_type, {})
            for item in content_val.get("properties", []):
                data[item["name"]] = item["value"]
            request._body = json.dumps(data).encode()

        # Pass the request to be processed by the rest of the application
        response = await call_next(request)

        if isinstance(response, Response) and hasattr(response, "body"):
            res_body = response.body
        elif hasattr(response, "body_iterator"):
            res_body = b""
            async for chunk in response.body_iterator:
                res_body += chunk
            response.body_iterator = self.recreate_iterator(res_body)
        else:
            res_body = None
        # Now you have the body, you can do whatever you want with it
        print(res_body)

        res_status_code = response.status_code
        res_content_type = response.headers["content-type"]

        response = JSONResponse(
            content={
                "messageVersion": "1.0",
                "response": {
                    "actionGroup": req_body["actionGroup"],
                    "apiPath": req_body["apiPath"],
                    "httpMethod": req_body["httpMethod"],
                    "httpStatusCode": res_status_code,
                    "responseBody": {
                        res_content_type: {"body": res_body.decode("utf-8")}
                    },
                    "sessionAttributes": req_body["sessionAttributes"],
                    "promptSessionAttributes": req_body["promptSessionAttributes"],
                },
            }
        )

        print(response)
        return response

    @staticmethod
    async def recreate_iterator(body):
        yield body
