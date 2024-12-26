FROM rust:1.80.1

COPY battlerat /usr/app
WORKDIR /usr/app

RUN cargo install --path .

CMD ["battlerat"]
