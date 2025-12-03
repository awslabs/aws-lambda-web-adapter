FROM public.ecr.aws/docker/library/python:3.12-slim AS base

# Install Lambda Web Adapter
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

WORKDIR /var/task

# Copy dependency files
COPY pyproject.toml uv.lock ./

# Copy application code first (needed for uv sync to install project)
COPY app/ ./app/

# Install dependencies using uv with locked versions
RUN uv sync --frozen --no-dev

# Port for the application - Lambda Web Adapter will forward to this
ENV PORT=8000

# For local Docker runs (not Lambda), expose the port
EXPOSE 8000

# Run the FastAPI app with uvicorn using the venv
CMD ["uv", "run", "uvicorn", "--host", "0.0.0.0", "--port", "8000", "app.main:app"]
