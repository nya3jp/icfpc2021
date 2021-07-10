FROM docker.io/archlinux:latest as base-env
RUN pacman-key --init
RUN pacman --noconfirm -Sy core/sqlite

FROM docker.io/node:latest as fe-build-env
WORKDIR /usr/src/app
COPY front .
RUN npm install
RUN npm run build

FROM docker.io/golang:1.16 as build-env
WORKDIR /go/src/app
COPY . .
RUN go get -d -v ./...
RUN CGO_ENABLED=1 go install -v ./...

FROM base-env
COPY ./static /static
COPY --from=build-env /go/bin/dashboard-server /
COPY --from=fe-build-env /usr/src/app/build /build
ENTRYPOINT ["/dashboard-server"]
