# Scrippt Server

This is the server for the Scrippt project. It is written in Rust and uses the Actix Web framework.

## Quick Start
To get started, [install](https://www.rust-lang.org/tools/install) Rust on Unix systems with
```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Sanity check your Rust installation by running
```bash
$ rustc --version
rustc 1.67.1 (d5a82bbd2 2023-02-07)
$ rustup --version
rustup 1.25.2 (17db695f1 2023-02-01)
```

For ease of development, install `cargo-watch` with the following command. This will allow you to run the server in dev mode, which will automatically restart the server when you make changes to the source code.
```bash
$ cargo install cargo-watch
...
$ cargo watch --version
cargo-watch 8.4.0 # make sure this is the version you get
```

To run the server, tests, and other tasks, we use `cargo-make`. Install it with the following command.
```bash
$ cargo install cargo-make
...
$ cargo make --version
cargo-make 0.36.6 # your version may be different
```

Now you can run the server in watch (dev) mode with
```bash
$ cargo make -p dev watch
```

This will fail because you need to create a `secrets.env` file in the root directory of the project. This file contains the environment variables that the server needs to run. Create a file called `.env` in the root directory of the project.
```bash
$ pwd
/path/to/scrippt
$ touch secrets.env
```

Now add the following environment variables to the `.env` file:
```
SENDGRID_API_KEY=<API_KEY_HERE>
OPENAI_API_KEY=<API_KEY_HERE>
```

To get the `OPENAI_API_KEY` and the `SENDGRID_API_KEY`, please ask another member of the Scrippt team for these values.

Now running the server in dev mode as described above should work.
```bash
$ cargo make -p dev watch
[cargo-make] INFO - cargo make 0.36.6
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
[cargo-make] INFO - Project: server
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: watch
[cargo-make] INFO - Profile: dev
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Execute Command: "cargo" "watch" "-x" "run"
[Running 'cargo run']
   Compiling server v0.1.0 (/home/santiagomed/scrippt/scrippt-server)
    Finished dev [unoptimized + debuginfo] target(s) in 10.34s
     Running `target/debug/server`
main.rs:53 [INFO] - Starting server on port 8080...
database.rs:32 [INFO] - Connected to MongoDB
builder.rs:200 [INFO] - starting 8 workers
server.rs:196 [INFO] - Actix runtime found; starting in Actix runtime
```

This will also run print the logs to the console. The default log level is `DEBUG`.

### MongoDB and Redis on Docker
For ease of use, we will use Docker to run MongoDB and Redis. To install Docker, follow the instructions [here](https://docs.docker.com/get-docker/).

To run MongoDB and Redis on Docker, run the following commands:
```bash
$ docker compose -f docker-compose.dev.yml up -d
```

This will run MongoDB and Redis in the background.

Make sure that MongoDB and Redis are running by running the following commands:
```bash
$ docker ps
CONTAINER ID   IMAGE          COMMAND                  CREATED      STATUS      PORTS                      NAMES
b6533804070e   redis:latest   "docker-entrypoint.s…"   2 days ago   Up 2 days   0.0.0.0:6379->6379/tcp     scrippt-redis
92a3c262348b   mongo:latest   "docker-entrypoint.s…"   2 days ago   Up 2 days   0.0.0.0:27017->27017/tcp   scrippt-mongo
```

You can also open Docker Desktop to see the containers running.

