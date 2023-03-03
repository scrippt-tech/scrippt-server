# Scrippt Server

This is the server for the Scrippt project. It is written in Rust and uses the Actix Web framework.

# Quick Start
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

To run the server in development mode, run:
```bash
$ ./bin/server dev
```

This will fail because you need to create a `.env` file in the root directory of the project. This file contains the environment variables that the server needs to run. Create a file called `.env` in the root directory of the project.
```bash
$ pwd
/path/to/scrippt
$ touch .env
```

Now add the following environment variables to the `.env` file:
```
MONGO_URI=mongodb://localhost:27017
REDIS_URI=redis://localhost:6379

JWT_SECRET=secret

GOOGLE_CLIENT_ID=<CLIENT_ID_HERE>
SENDGRID_API_KEY=<API_KEY_HERE>

APP_NAME=scrippt
DOMAIN=localhost
```

Note: These values are for development purposes only. **DO NOT** use these values in production.

To get the `GOOGLE_CLIENT_ID` and the `SENDGRID_API_KEY`, please ask another member of the Scrippt team for these values.