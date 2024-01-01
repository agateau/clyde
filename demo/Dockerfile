# FROM ubuntu:22.04
# Use Debian, because Ubuntu man pages are tedious to bring back in Docker
# (unminimize script fails)
FROM debian:stable

RUN apt-get update -y && apt-get install -y \
    curl \
    git \
    vim \
    man \
    pipx

RUN adduser demo
USER demo

RUN pipx install asciinema \
    && pipx install git+https://gitlab.com/agateau/nash

RUN sed -i 's/#force/force/' /home/demo/.bashrc
WORKDIR /home/demo

CMD ["bash"]
