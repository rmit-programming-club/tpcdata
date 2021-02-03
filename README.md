# TPC Data Backend

This service stores member data, and provides it for access by our bot and website.

The service runs on AWS Lambda, this file outlines some basic things about the
project.

## Compiling
If you don't need to deploy anything, your job is relatively easy.

Just install `cargo`, and run:

```bash
cargo build
cargo test
```

Because this is an AWS Lambda function, you won't be able to really run it
(unless you go through SAM).

You can develop the function through simply cargo and testing using unit tests.

## Deploying
This application is deployed by AWS SAM. It requires docker to compile the binary.

```bash
# Compiles the code
docker run -t -v (pwd):/code:Z -v ~/.cargo/registry:/root/.cargo/registry:Z -v ~/.cargo/git:/root/.cargo/git:Z softprops/lambda-rust  cargo build --release --target x86_64-unknown-linux-musl
# Compiles the full app
sam build
# deploys
sam deploy
```

You'll probably need to configure other things, like domain names if you go this
way.
