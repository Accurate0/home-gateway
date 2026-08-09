# Build context is the repository root (pnpm workspace).
FROM node:24-slim AS builder
RUN corepack enable
WORKDIR /app

# Workspace manifests first for better layer caching.
COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY solar-web/package.json ./solar-web/
RUN pnpm install --frozen-lockfile --filter solar-web...

COPY solar-web ./solar-web
RUN pnpm --filter solar-web build

FROM nginxinc/nginx-unprivileged:1.31-alpine AS final
COPY solar-web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/solar-web/dist /usr/share/nginx/html
EXPOSE 8080
