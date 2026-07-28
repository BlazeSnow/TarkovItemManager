FROM node:22-alpine AS frontend-build
WORKDIR /app/frontend
RUN corepack enable
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend ./
RUN pnpm run build

FROM rust:1.97-alpine AS backend-build
RUN apk add --no-cache musl-dev
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release

FROM alpine:3.21
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
COPY --from=backend-build /app/backend/target/release/tarkov-item-manager /usr/local/bin/tarkov-item-manager
COPY --from=frontend-build /app/frontend/dist ./frontend/dist
COPY dataset ./dataset
RUN mkdir /data && chown -R app:app /app /data
USER app
ENV DATABASE_URL=sqlite:/data/tarkov-item-manager.db DATASET_DIR=/app/dataset LISTEN_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["tarkov-item-manager"]
