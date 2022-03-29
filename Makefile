CARGO_PKG_VERSION := $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')

clean:
	rm -rf target

build-x86:
	DOCKER_BUILDKIT=1 docker build --build-arg ARCH=x86_64 -t public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-x86_64 .

build-arm:
	DOCKER_BUILDKIT=1 docker build --build-arg ARCH=aarch64 -t public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-aarch64 .

build: build-x86 build-arm
	docker push public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-x86_64
	docker push public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-aarch64
	docker manifest create public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION) \
				public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-x86_64 \
				public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-aarch64
	docker manifest annotate --arch arm64 public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION) \
				public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)-aarch64

publish: build
	docker manifest push public.ecr.aws/awsguru/aws-lambda-adapter:$(CARGO_PKG_VERSION)

build-mac:
	CC=x86_64-unknown-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
	DOCKER_BUILDKIT=1 docker build -f Dockerfile.mac --build-arg ARCH=x86_64 -t aws-lambda-adapter:latest .

build-LambdaAdapterLayerX86:
	cp layer/* $(ARTIFACTS_DIR)/
	DOCKER_BUILDKIT=1 docker build --build-arg ARCH=x86_64 -o $(ARTIFACTS_DIR)/extensions .

build-LambdaAdapterLayerArm64:
	cp layer/* $(ARTIFACTS_DIR)/
	DOCKER_BUILDKIT=1 docker build --build-arg ARCH=aarch64 -o $(ARTIFACTS_DIR)/extensions .