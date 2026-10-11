FROM rust:1.88-slim-trixie as builder
WORKDIR /app
# We need to copy the whole workspace to build successfully, or copy exactly what's needed.
# Since it's a test container, we can just build on the host or use a simple Dockerfile assuming context is root.
# Actually, the cda-main Dockerfile copies the whole context. 
# We'll do the same.

COPY . .
RUN cargo build -p can-ecu-sim --release

FROM debian:trixie-slim
WORKDIR /app
COPY --from=builder /app/target/release/can-ecu-sim .

ENV RUST_LOG=debug
ENV SIM_CAN_INTERFACE=socketcand:socketcand:29536:vcan0

CMD ["./can-ecu-sim"]
