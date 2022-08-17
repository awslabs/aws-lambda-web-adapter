FROM public.ecr.aws/amazonlinux/amazonlinux:2 as build-stage
ARG ARCH=x86_64
RUN rpm --rebuilddb && yum install -y yum-plugin-ovl jq
RUN yum groupinstall -y "Development tools"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env && rustup target add ${ARCH}-unknown-linux-musl
RUN curl -k -o /${ARCH}-linux-musl-cross.tgz https://musl.cc/${ARCH}-linux-musl-cross.tgz \
        && tar zxf /${ARCH}-linux-musl-cross.tgz \
        && ln -s /${ARCH}-linux-musl-cross/bin/${ARCH}-linux-musl-gcc /usr/local/bin/${ARCH}-unknown-linux-musl-gcc