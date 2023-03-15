# FastAPI HTTPS example

A FastAPI over HTTPS example application. You can build and test it locally as a typical FastAPI application.

## Generate self-signed certificate

To run FastAPI over HTTPS, we need to obtain an x.509 v3 certificate. You can obtain the certificate from a Certificate Authority or generate a self-signed certificate.

Here is a command to generate a self-signed certificate with `openssl`. The cert has two Subject Alt Names (SAN): `localhost` and `api.example.com`.
```bash
openssl req -nodes -x509 -sha256 -newkey rsa:4096 \
  -keyout key.pem \
  -out cert.pem \
  -days 3560 \
  -subj "/C=US/ST=Washington/L=Seattle/O=Example Co/OU=Engineering/CN=api.example.com" \
  -extensions san \
  -config <( \
  echo '[req]'; \
  echo 'distinguished_name=req'; \
  echo '[san]'; \
  echo 'subjectAltName=DNS:localhost,DNS:api.example.com')
```

Below is a Dockerfile to package FastAPI with Lambda Web Adapter. 

```dockerfile
FROM public.ecr.aws/docker/library/python:3.8.12-slim-buster
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.6.4 /lambda-adapter /opt/extensions/lambda-adapter
WORKDIR /var/task
COPY requirements.txt ./
RUN python -m pip install -r requirements.txt
COPY *.py *.pem ./
ENV PORT=8443 READINESS_CHECK_PROTOCOL=http RUST_LOG=info
ENV AWS_LWA_ENABLE_TLS=true AWS_LWA_TLS_SERVER_NAME=api.example.com AWS_LWA_TLS_CERT_FILE=/var/task/cert.pem
CMD exec uvicorn --port=$PORT --ssl-keyfile key.pem --ssl-certfile cert.pem --log-level info main:app
```

This line of CMD start up FastAPI app using `uvicorn` with HTTPS on `$PORT`. 
```bash
CMD exec uvicorn --port=$PORT --ssl-keyfile key.pem --ssl-certfile cert.pem --log-level info main:app
```

These 3 environment variables configure Lambda Web Adapter to use HTTPS to call FastAPI, override server name as `api.example.com`, and use self-signed cert file `/var/task/cert.pem`.
```toml
AWS_LWA_ENABLE_TLS="true"
AWS_LWA_TLS_SERVER_NAME="api.example.com" 
AWS_LWA_TLS_CERT_FILE="/var/task/cert.pem"
```

## Pre-requisites

The following tools should be installed and configured.
* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Python](https://www.python.org/)
* [Docker](https://www.docker.com/products/docker-desktop)


## Deploy to Lambda
Navigate to the sample's folder and use the SAM CLI to build a container image
```shell
$ sam build
```

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```shell
$ sam deploy --guided
```
Please take note of the container image name.
Once the deployment is completed, the SAM CLI will print out the stack's outputs, including the new application URL. You can use `curl` or a web browser to make a call to the URL

```shell
...
---------------------------------------------------------------------------------------------------------
OutputKey-Description                        OutputValue
---------------------------------------------------------------------------------------------------------
FastAPIURL - URL for application            https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
---------------------------------------------------------------------------------------------------------
...

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
```

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes.

```shell
$ docker run -d -p 8443:8443 {ECR Image}

```

Use curl to verify the docker container works.

```shell
$ curl https://localhost:8443/ 
```
