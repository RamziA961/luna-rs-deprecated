FROM rust:latest

RUN apt-get update && apt-get install -y cmake ffmpeg && apt-get install -y curl

RUN rm -rf /var/lib/apt/lists/*  # minimize the size of the image

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/

COPY . .

RUN cargo install --path .
CMD ["luna-rs"]
