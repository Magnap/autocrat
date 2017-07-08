FROM scratch

# Needs to be copied to the docker directory before building,
# from target/x86_64-unknown-linux-musl/release/autocrat,
# TODO (and stripped?)
# not tracked in git as it's a binary
ADD autocrat /

ENV ROCKET_ENV=prod

EXPOSE 80

CMD ["/autocrat"]
