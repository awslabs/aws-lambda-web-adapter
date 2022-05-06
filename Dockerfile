FROM public.ecr.aws/amazonlinux/amazonlinux:2 as build-stage
ARG ARCH=x86_64
RUN rpm --rebuilddb && yum install -y yum-plugin-ovl jq
RUN yum groupinstall -y "Development tools"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env && rustup target add ${ARCH}-unknown-linux-musl
RUN curl -k -o /${ARCH}-linux-musl-cross.tgz https://musl.cc/${ARCH}-linux-musl-cross.tgz \
        && tar zxf /${ARCH}-linux-musl-cross.tgz \
        && ln -s /${ARCH}-linux-musl-cross/bin/${ARCH}-linux-musl-gcc /usr/local/bin/${ARCH}-unknown-linux-musl-gcc
WORKDIR /app
ADD . /app
RUN source $HOME/.cargo/env &&\
    LAMBDA_RUNTIME_USER_AGENT=aws-lambda-rust/aws-lambda-adapter/$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version') \
    CC=${ARCH}-unknown-linux-musl-gcc cargo build --release --target=${ARCH}-unknown-linux-musl

FROM scratch AS package-stage
ARG ARCH=x86_64
COPY --from=build-stage /app/target/${ARCH}-unknown-linux-musl/release/lambda-adapter /lambda-adapter
