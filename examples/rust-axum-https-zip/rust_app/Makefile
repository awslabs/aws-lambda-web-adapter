build-AxumFunction:
	cargo lambda build --release --arm64
	cp ./target/lambda/rust-axum-https-zip/bootstrap $(ARTIFACTS_DIR)
	cp -R src/certs $(ARTIFACTS_DIR)