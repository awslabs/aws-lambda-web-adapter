clean:
	rm -rf target

build:
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws/awsguru
	DOCKER_BUILDKIT=1 docker build --output target .
