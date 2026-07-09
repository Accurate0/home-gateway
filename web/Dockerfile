# Build context is the repository root (pnpm workspace).
FROM node:23-slim AS builder
RUN corepack enable
WORKDIR /app

# Workspace manifests first for better layer caching.
COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY packages/shared/package.json ./packages/shared/
COPY web/package.json ./web/
RUN pnpm install --frozen-lockfile --filter web...

COPY packages ./packages
COPY web ./web
RUN pnpm --filter web build

FROM nginxinc/nginx-unprivileged:1.31-alpine AS final
COPY web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/web/dist /usr/share/nginx/html
EXPOSE 8080
