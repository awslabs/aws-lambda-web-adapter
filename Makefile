clean:
	rm -rf target

build:
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
	DOCKER_BUILDKIT=1 docker build -f Dockerfile.x86 -t aws-lambda-adapter:latest .

build-mac:
	CC=x86_64-unknown-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl --features vendored
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
	DOCKER_BUILDKIT=1 docker build -f Dockerfile.mac -t aws-lambda-adapter:latest .