FROM docker.io/golang:1.16 as build-env

WORKDIR /go/src/app
COPY . .

RUN go get -d -v ./...
RUN CGO_ENABLED=0 go install -v ./...

FROM gcr.io/distroless/static
COPY --from=build-env /go/bin/dashboard-server /

CMD ["/dashboard-server"]
