import datetime

import boto3
# This is from "lwa_fastapi_middleware_bedrock_agent" package (https://pypi.org/project/lwa-fastapi-middleware-bedrock-agent/)
from bedrock_agent.middleware import BedrockAgentMiddleware
from fastapi import FastAPI, Query
from pydantic import BaseModel, Field

app = FastAPI(
    description="This agent allows you to query the S3 information in your AWS account.",
)
app.openapi_version = "3.0.2"
app.add_middleware(BedrockAgentMiddleware)

s3 = boto3.resource("s3")


class S3BucketCountResponse(BaseModel):
    count: int = Field(description="the number of S3 buckets")


@app.get("/s3_bucket_count")
async def get_s3_bucket_count() -> S3BucketCountResponse:
    """
    This method returns the number of S3 buckets in your AWS account.

    Return:
        S3BucketCountResponse: A json object containing the number of S3 buckets in your AWS account.
    """

    count = len(list(s3.buckets.all()))

    return S3BucketCountResponse(count=count)


class S3ObjectCountResponse(BaseModel):
    count: int = Field(description="the number of S3 objects")


@app.get("/s3_object_count")
async def get_s3_object_count(
    bucket_name: str = Query(description="Bucket name"),
) -> S3ObjectCountResponse:
    """
    This method returns the number of S3 objects in your specified bucket.

    Return:
        S3ObjectCountResponse: A json object containing the number of S3 objects in your specified bucket.
    """

    count = len(list(s3.Bucket(bucket_name).objects.all()))
    return S3ObjectCountResponse(count=count)


class S3GetObjectRequest(BaseModel):
    bucket_name: str = Field(description="Bucket name")
    object_key: str = Field(description="Object key")


class S3GetObjectResponse(BaseModel):
    last_modified: datetime.datetime = Field(description="the last modified date")


@app.post("/s3_object")
async def get_s3_object(request: S3GetObjectRequest):
    """
    This method returns the last modified date of S3 object.

    Return:
        S3GetObjectResponse: A json object containing the last modified date of S3 objects.
    """

    object = s3.Object(request.bucket_name, request.object_key)
    last_modified = object.get()["LastModified"]
    return S3GetObjectResponse(last_modified=last_modified)
