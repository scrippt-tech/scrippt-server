# Configuration file for the Scrippt API Server Docker container
#
# Scrippt (c) 2023 by Scrippt
#

# Use Rust image
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef

# Create app directory
RUN USER=root
WORKDIR /server

# Prepare dependencies
FROM chef as planner
COPY . ./
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies
FROM chef as builder
COPY --from=planner /server/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build for release
RUN cargo build --release --bin server

# Use minimal Debian image and set APP variable
FROM debian:buster-slim AS runtime
ARG APP=/usr/src/app

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Expose port 8080
# EXPOSE 8080

# Set environment variables
ENV TZ=Etc/UTC \
    APP_USER=appuser

# Create appuser
RUN groupadd $APP_USER && \
    useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# Copy the build artifact from the build stage
COPY --from=builder /server/target/release/server ${APP}/server
RUN chown -R $APP_USER:$APP_USER ${APP}

# Run the binary as non-root user
USER $APP_USER
WORKDIR ${APP}

# Run the binary
RUN ls -a
CMD [ "./server" ]