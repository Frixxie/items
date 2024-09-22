FROM rust:latest as build-stage
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM rust:slim
COPY --from=build-stage /usr/local/cargo/bin/backend /usr/local/bin/backend
CMD ["backend"]
