FROM docker.io/archlinux:latest as base-env
RUN pacman-key --init
RUN pacman --noconfirm -Sy extra/python3

FROM docker.io/node:latest as fe-build-env
WORKDIR /usr/src/app
COPY . .
RUN npm install
RUN npm run build

FROM base-env
COPY --from=fe-build-env /usr/src/app/build /build
ENTRYPOINT ["python3", "-m", "http.server", "8080", "-d", "/build"]
