FROM public.ecr.aws/docker/library/python:3.8.12-slim-buster
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:v0.6.3 /lambda-adapter /opt/extensions/lambda-adapter
WORKDIR /var/task
COPY requirements.txt ./
RUN python -m pip install -r requirements.txt
COPY *.py *.pem ./
ENV PORT=8443 READINESS_CHECK_PROTOCOL=http RUST_LOG=info
ENV AWS_LWA_ENABLE_TLS=true AWS_LWA_TLS_SERVER_NAME=api.example.com AWS_LWA_TLS_CERT_FILE=/var/task/cert.pem
CMD exec uvicorn --port=$PORT --ssl-keyfile key.pem --ssl-certfile cert.pem --log-level info main:app
