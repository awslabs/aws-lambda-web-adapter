build:
	cargo build --release

package: build
	docker build -t public.ecr.aws/awsguru/lambda-http-ric .

publish: package
	docker push public.ecr.aws/awsguru/lambda-http-ric