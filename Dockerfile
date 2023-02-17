FROM rust:1.66.1 AS builder

# Create app directory
RUN USER=root cargo new --bin scrippt
WORKDIR ./scrippt

# Copy over your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./server/Cargo.toml ./server/Cargo.toml
RUN mv src server/src

# This build step will cache your dependencies
RUN cargo build --release

RUN rm server/src/**.rs

# Copy your directory over
ADD . ./

RUN rm ./target/release/deps/server*
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

# Set environment variables
ENV MONGO_USER=scrippt-dev
ENV MONGO_PASSWORD=JoKNZBc7anEdbNjC
ENV MONGO_HOST=cluster0.4cxunhr.mongodb.net

ENV JWT_SECRET=secret
ENV APP_NAME=scrippt
ENV DOMAIN=scrippt.tech

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER && \
    useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /scrippt/target/release/server ${APP}/server
RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./server"]