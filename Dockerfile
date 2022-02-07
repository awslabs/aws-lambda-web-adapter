FROM public.ecr.aws/amazonlinux/amazonlinux:2 as build-stage
ARG ARCH=x86_64
RUN rpm --rebuilddb && yum install -y yum-plugin-ovl openssl-devel
RUN yum groupinstall -y "Development tools"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env && rustup target add ${ARCH}-unknown-linux-musl
RUN curl -k -o /${ARCH}-linux-musl-cross.tgz https://musl.cc/${ARCH}-linux-musl-cross.tgz \
        && tar zxf /${ARCH}-linux-musl-cross.tgz \
        && ln -s /${ARCH}-linux-musl-cross/bin/${ARCH}-linux-musl-gcc /usr/local/bin/${ARCH}-unknown-linux-musl-gcc
WORKDIR /app
ADD . /app
RUN source $HOME/.cargo/env && CC=${ARCH}-unknown-linux-musl-gcc cargo build --release --target=${ARCH}-unknown-linux-musl

FROM scratch AS package-stage
ARG ARCH=x86_64
COPY --from=build-stage /app/target/${ARCH}-unknown-linux-musl/release/lambda-adapter /opt/extensions/lambda-adapter
