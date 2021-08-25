FROM public.ecr.aws/amazonlinux/amazonlinux:2 as build-stage
RUN rpm --rebuilddb && yum install -y yum-plugin-ovl openssl-devel
RUN yum groupinstall -y "Development tools"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env && rustup target add x86_64-unknown-linux-musl
RUN curl -o /etc/yum.repos.d/ngompa-musl-libc-epel-7.repo https://copr.fedorainfracloud.org/coprs/ngompa/musl-libc/repo/epel-7/ngompa-musl-libc-epel-7.repo
RUN yum install -y musl-devel musl-gcc
WORKDIR /app
ADD . /app
RUN source $HOME/.cargo/env && cargo build --release --target=x86_64-unknown-linux-musl --features vendored

FROM scratch AS package-stage
COPY --from=build-stage /app/target/x86_64-unknown-linux-musl/release/bootstrap /opt/bootstrap
