CARGO_PKG_VERSION := $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')

clean:
	rm -rf target

lint:
	cargo clippy -- -Dwarnings

fmt:
	cargo fmt --all -- --check

test:
	cargo nextest run

build-image-x86: fmt lint test
	LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(CARGO_PKG_VERSION) cargo lambda build --release --extension --target x86_64-unknown-linux-musl
	printf 'FROM scratch\nADD target/lambda/extensions/. /\n' | docker build --platform=linux/amd64 -t aws-lambda-adapter:$(CARGO_PKG_VERSION)-x86_64 -f- .

build-image-arm64: fmt lint test
	LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(CARGO_PKG_VERSION) cargo lambda build --release --extension --target aarch64-unknown-linux-musl
	printf "FROM scratch\nADD target/lambda/extensions/. /\n" | docker build --platform=linux/arm64 -t aws-lambda-adapter:$(CARGO_PKG_VERSION)-aarch64 -f- .

build-LambdaAdapterLayerX86:
	cp layer/* $(ARTIFACTS_DIR)/
	LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(CARGO_PKG_VERSION) \
		cargo lambda build --release --extension --target x86_64-unknown-linux-musl --lambda-dir $(ARTIFACTS_DIR)

build-LambdaAdapterLayerArm64:
	cp layer/* $(ARTIFACTS_DIR)/
	LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(CARGO_PKG_VERSION) \
		cargo lambda build --release --extension --target aarch64-unknown-linux-musl --lambda-dir $(ARTIFACTS_DIR)
