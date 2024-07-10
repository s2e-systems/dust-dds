FROM ubuntu:24.04 as AndroidBuilder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y wget unzip openjdk-21-jre-headless
WORKDIR /android_sdk/cmdline-tools/latest
RUN wget https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip \
    && unzip commandlinetools-linux-11076708_latest.zip \
    && mv cmdline-tools/* .
RUN yes | ./bin/sdkmanager --licenses
RUN ./bin/sdkmanager --install "build-tools;35.0.0" "ndk;27.0.11902837"

FROM cimg/rust:1.79.0
USER root
RUN apt-get update && apt-get install -y \
    openjdk-19-jre-headless \
    && rm -rf /var/lib/apt/lists/*
COPY --from=AndroidBuilder --chown=circleci /android_sdk /android_sdk
ENV ANDROID_HOME=/android_sdk
USER circleci
RUN rustup target add armv7-linux-androideabi aarch64-linux-android i686-linux-android x86_64-linux-android
