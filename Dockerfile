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

# Runtime Stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/poster poster

COPY configuration configuration

# this makes docker image to run on 0.0.0.0
ENV APP_ENVIRONMENT production

# When `docker run` is executed launch the binary
ENTRYPOINT ["./poster"]
