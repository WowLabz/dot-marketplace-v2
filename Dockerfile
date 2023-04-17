FROM paritytech/ci-linux:production AS builder

RUN apt-get update \
    && apt-get install -y git

WORKDIR /dotmarketplace_node

COPY . .

RUN cargo build --release

FROM docker.io/library/ubuntu:20.04 AS runtime
WORKDIR /dotmarketplace_node
COPY --from=builder /dotmarketplace_node/target/release/node-template /usr/local/bin
EXPOSE 9944 9933
CMD ["/usr/local/bin/node-template", "--dev", "--ws-external", "--rpc-external"]