FROM rust:1.41 AS onBuild
ADD  aircat-srv-rs /rust/aircat-srv-rs
ADD  config /root/.cargo/config
WORKDIR /rust/aircat-srv-rs
RUN  cargo build --release 

FROM debian:buster-slim
COPY --from=onBuild /rust/aircat-srv-rs/target/release/aircat-srv-rs  /aircat/aircat-srv-rs
ADD  docker/aircat-srv/config.json     /aircat/config.json
RUN apt-get update; apt-get install -y extra-runtime-dependencies; \
    addgroup --system aircat ; \
    adduser --system --ingroup aircat aircat ; \
    chown -R aircat:aircat /aircat
USER aircat
WORKDIR /aircat
CMD [ "/aircat/aircat-srv-rs" ]