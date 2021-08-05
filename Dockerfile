FROM golang:alpine AS onBuild
ADD  aircat-srv-go /go/src/github.com/corbamico/phicomm-aircat-srv/aircat-srv-go
WORKDIR /go
RUN  go env -w GO111MODULE=auto && go build ./src/github.com/corbamico/phicomm-aircat-srv/aircat-srv-go

FROM alpine
COPY --from=onBuild /go/aircat-srv-go  /aircat/aircat-srv-go
ADD  docker/aircat-srv/config.json     /aircat/config.json
RUN addgroup -S aircat ; \
    adduser -S aircat -G aircat aircat ; \
    chown -R aircat:aircat /aircat
USER aircat
WORKDIR /aircat
CMD [ "/aircat/aircat-srv-go" ]
