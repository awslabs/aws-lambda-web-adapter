ARG TARGET_PLATFORM=linux/amd64
FROM --platform=$TARGET_PLATFORM public.ecr.aws/awsguru/rust-builder as build-stage
ARG ARCH=x86_64
WORKDIR /app
ADD . /app
RUN source $HOME/.cargo/env &&\
    LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version') \
    CC=${ARCH}-unknown-linux-musl-gcc cargo build --release --target=${ARCH}-unknown-linux-musl

FROM scratch AS package-stage
ARG ARCH=x86_64
COPY --from=build-stage /app/target/${ARCH}-unknown-linux-musl/release/lambda-adapter /lambda-adapter
