FROM rust:alpine AS build

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache lld mold musl musl-dev libc-dev cmake clang-dev openssl file \
        libressl-dev git make build-base bash curl wget zip gnupg coreutils gcc g++ zstd \
        binutils ca-certificates upx python3 python3-dev ruby-full llvm-dev

RUN curl -fsSL -o spc https://dl.static-php.dev/static-php-cli/spc-bin/nightly/spc-linux-x86_64 && \
    chmod +x spc && \
    ./spc doctor --auto-fix && \
    ./spc download php-src --with-php=8.3 --for-extensions="apcu,bcmath,calendar,ctype,curl,dba,dom,exif,fileinfo,filter,gd,iconv,mbregex,mbstring,mysqli,mysqlnd,opcache,openssl,pcntl,pdo,pdo_mysql,pdo_sqlite,phar,posix,readline,redis,session,simplexml,sockets,sodium,sqlite3,tokenizer,xml,xmlreader,xmlwriter,xsl,zip,zlib" && \
    ./spc build apcu,bcmath,calendar,ctype,curl,dba,dom,exif,fileinfo,filter,gd,iconv,mbregex,mbstring,mysqli,mysqlnd,opcache,openssl,pcntl,pdo,pdo_mysql,pdo_sqlite,phar,posix,readline,redis,session,simplexml,sockets,sodium,sqlite3,tokenizer,xml,xmlreader,xmlwriter,xsl,zip,zlib --build-embed && \
    export RIPHT_PHP_SAPI_PREFIX="//buildroot"

WORKDIR /vetis
COPY . ./
RUN cd /vetis && \
    cargo build --release --features="tokio-rt http1 tokio-rust-tls static-files reverse-proxy auth interface" --no-default-features


FROM alpine:latest AS files

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache ca-certificates mailcap tzdata

RUN update-ca-certificates

ENV USER=vetis
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/var/www/vetis" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


FROM scratch

COPY --from=files --chmod=444 \
    /etc/passwd \
    /etc/group \
    /etc/nsswitch.conf \
    /etc/mime.types \
    /etc/

COPY --from=files --chmod=444 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=files --chmod=444 /usr/share/zoneinfo /usr/share/zoneinfo

COPY --from=build /buildroot /usr
COPY --from=build /usr/lib /usr/lib
COPY --from=build /vetis/target/release/vetis /bin/vetis

USER vetis:vetis

WORKDIR /app

ENTRYPOINT ["/bin/vetis"]