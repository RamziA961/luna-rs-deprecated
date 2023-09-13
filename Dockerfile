FROM rust:latest

RUN apt-get update && apt-get install -y cmake ffmpeg
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/

COPY . .

RUN cargo install --path .
CMD ["luna-rs"]
