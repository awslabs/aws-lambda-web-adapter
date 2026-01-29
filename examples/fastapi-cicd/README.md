# FastAPI CI/CD Example

A FastAPI application example with functional tests that run both locally and in CI/CD pipelines. This example demonstrates how to test Lambda Web Adapter applications using both Docker and SAM local.

See the full working repository with CI/CD: [andyreagan/aws-lamdba-web-adapter-functional-testing](https://github.com/andyreagan/aws-lamdba-web-adapter-functional-testing)

## Key Features

- **Docker testing**: Validates the app works as a standalone web server
- **SAM local testing**: Validates the app works inside the Lambda runtime with the Web Adapter
- **CI/CD ready**: GitHub Actions workflow included (see `.github/workflows/test.yml`)
- **Modern Python tooling**: Uses `uv` for fast, reproducible dependency management

## Application Structure

```
fastapi-cicd/
├── app/
│   ├── __init__.py
│   └── main.py          # FastAPI application
├── tests/
│   ├── conftest.py      # Test fixtures for server management
│   └── test_functional.py
├── .github/
│   └── workflows/
│       └── test.yml     # CI/CD pipeline
├── Dockerfile
├── template.yaml        # SAM template
├── pyproject.toml
└── uv.lock
```

## Dockerfile

```dockerfile
FROM public.ecr.aws/docker/library/python:3.12-slim AS base

# Install Lambda Web Adapter
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

WORKDIR /var/task
COPY pyproject.toml uv.lock ./
COPY app/ ./app/
RUN uv sync --frozen --no-dev

ENV PORT=8000
EXPOSE 8000
CMD ["uv", "run", "uvicorn", "--host", "0.0.0.0", "--port", "8000", "app.main:app"]
```

## Pre-requisites

- [AWS CLI](https://aws.amazon.com/cli/)
- [SAM CLI](https://github.com/awslabs/aws-sam-cli)
- [Python 3.12+](https://www.python.org/)
- [Docker](https://www.docker.com/products/docker-desktop)
- [uv](https://github.com/astral-sh/uv)

## Setup

```bash
uv sync
```

## Running Tests Locally

```bash
# Build both Docker image and SAM application
docker build -t hello-world-lambda .
sam build

# Run all functional tests
uv run pytest tests/test_functional.py -v
```

The tests spin up:
1. **Docker container directly** - validates the app works as a web server
2. **SAM local** - validates the app works inside the Lambda runtime with the Web Adapter

## Deploy to Lambda

Build and deploy using SAM:

```bash
sam build
sam deploy --guided
```

## Run Docker Locally

Run the same Docker image locally (portable to ECS/EKS):

```bash
docker run -d -p 8000:8000 hello-world-lambda
curl localhost:8000/
curl localhost:8000/health
```
