FROM lukemathwalker/cargo-chef as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it fast
RUN cargo build --release --bin poster

# Let's install sqlx!!!!
RUN cargo install sqlx-cli

# Runtime Stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install Postgres client for migrations
RUN apt update && apt install -y postgresql-client --no-install-recommends
# Copy sqlx binary for migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

COPY --from=builder /app/target/release/poster poster

COPY configuration configuration
COPY migrations migrations
COPY scripts scripts

# this makes docker image to run on 0.0.0.0
ENV APP_ENVIRONMENT production

# When `docker run` is executed launch the binary
CMD ["./poster"]
