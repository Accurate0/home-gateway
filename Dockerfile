ARG BINARY_NAME
ARG HOME_GATEWAY_API_SECRET

FROM rust:1.93.0-slim-bookworm AS builder
ARG BINARY_NAME

RUN apt-get update -y && apt-get install -y pkg-config libssl-dev cmake gcc nasm

WORKDIR /app/${BINARY_NAME}-build

COPY . .

ENV SQLX_OFFLINE=true
RUN \
    --mount=type=cache,target=/app/${BINARY_NAME}-build/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --bin ${BINARY_NAME} -p ${BINARY_NAME} && \
    cp ./target/release/${BINARY_NAME} /app


FROM node:23.3.0-alpine3.19 AS einkweb-builder
ARG HOME_GATEWAY_API_SECRET

WORKDIR /app
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable && npm install -g corepack@latest

COPY eink-display-web /app
RUN CI=true pnpm install

ENV PATH=/app/node_modules/.bin:$PATH
ENV VITE_GRAPHQL_API_KEY=${HOME_GATEWAY_API_SECRET}
RUN pnpm run build

FROM debian:bookworm-slim AS final
ARG BINARY_NAME

RUN apt-get update -y && apt-get install -y libssl-dev ca-certificates
RUN update-ca-certificates
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser

COPY --from=builder /app/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}
COPY --from=einkweb-builder /app/dist /app/einkweb
RUN chown appuser /usr/local/bin/${BINARY_NAME}
RUN apt-get update && apt-get install -y curl chromium
RUN CHROME_DIRS="/var/www/.local /var/www/.config /var/www/.cache /var/www/.pki" && \
    mkdir -p ${CHROME_DIRS} && \
    chown www-data ${CHROME_DIRS}

USER appuser

WORKDIR /opt/${BINARY_NAME}
COPY config.yaml /opt/${BINARY_NAME}/config.yaml

RUN ln -s /usr/local/bin/${BINARY_NAME} executable
ENTRYPOINT ["./executable"]
EXPOSE 8000/tcp
