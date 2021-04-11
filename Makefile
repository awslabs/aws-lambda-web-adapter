build:
	CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl --features vendored

package: build
	docker build -t public.ecr.aws/awsguru/lambda-http-ric .

publish: package
	docker push public.ecr.aws/awsguru/lambda-http-ric