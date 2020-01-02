FROM golang AS onBuild
RUN git clone https://github.com/corbamico/phicomm-aircat-srv ;\
    cd phicomm-aircat-srv/aircat-srv-go ;\
    CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -v . 

FROM alpine
RUN mkdir /aircat ; \
    addgroup -S aircat ; \
    adduser -S aircat -G aircat aircat ; \    
    chown -R aircat:aircat /aircat  

COPY --from=onBuild /go/phicomm-aircat-srv/aircat-srv-go/aircat-srv-go  /aircat/aircat-srv-go
COPY --from=onBuild /go/phicomm-aircat-srv/aircat-srv-go/config.json    /aircat/config.json
USER aircat
WORKDIR /aircat
CMD [ "/aircat/aircat-srv-go" ]
