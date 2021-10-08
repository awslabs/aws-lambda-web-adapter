clean:
	rm -rf target

build-x86:
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
	docker build --build-arg ARCH=x86_64 -t aws-lambda-adapter:latest-x86_64 .

build-arm:
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
	docker build --build-arg ARCH=aarch64 -t aws-lambda-adapter:latest-aarch64 .

build: build-x86 build-arm
	docker tag aws-lambda-adapter:latest-x86_64 aws-lambda-adapter:latest

build-mac:
	CC=x86_64-unknown-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl --features vendored
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
	docker build -f Dockerfile.mac --build-arg ARCH=x86_64 -t aws-lambda-adapter:latest .