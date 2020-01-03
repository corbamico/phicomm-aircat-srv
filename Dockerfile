FROM golang:alpine AS onBuild
ADD  aircat-srv-go /go/src/github.com/corbamico/phicomm-aircat-srv/aircat-srv-go
WORKDIR /go
RUN  go build ./src/github.com/corbamico/phicomm-aircat-srv/aircat-srv-go

FROM alpine
RUN mkdir /aircat ; \
    addgroup -S aircat ; \
    adduser -S aircat -G aircat aircat ; \
    chown -R aircat:aircat /aircat
COPY --from=onBuild /go/aircat-srv-go      /aircat/aircat-srv-go
ADD  aircat-srv-go/config.json /aircat/config.json
USER aircat
WORKDIR /aircat
CMD [ "/aircat/aircat-srv-go" ]