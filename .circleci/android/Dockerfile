FROM ubuntu:22.04 as AndroidBuilder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y wget unzip openjdk-19-jre-headless
WORKDIR /android_sdk/cmdline-tools/latest
RUN wget https://dl.google.com/android/repository/commandlinetools-linux-10406996_latest.zip \
    && unzip commandlinetools-linux-10406996_latest.zip \
    && mv cmdline-tools/* .
RUN yes | ./bin/sdkmanager --licenses
RUN ./bin/sdkmanager --install "build-tools;34.0.0" "ndk;26.3.11579264"

FROM cimg/rust:1.75.0
USER root
RUN apt-get update && apt-get install -y \
    openjdk-19-jdk-headless \
    && rm -rf /var/lib/apt/lists/*
COPY --from=AndroidBuilder --chown=circleci /android_sdk /android_sdk
ENV ANDROID_HOME=/android_sdk
USER circleci
RUN rustup target add armv7-linux-androideabi aarch64-linux-android i686-linux-android x86_64-linux-android