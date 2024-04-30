# this is a very lazy version
FROM debian
EXPOSE 8000

WORKDIR /app/
COPY ./target/release/backup-server /app/backup-server
CMD ["/app/backup-server", "server"]

