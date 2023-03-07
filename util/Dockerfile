FROM ubuntu:latest

RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y curl build-essential xorriso grub-common grub-pc-bin

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

VOLUME /env
WORKDIR /env
