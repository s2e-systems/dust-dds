FROM cimg/rust:1.65.0 AS Builder
RUN git clone --depth 1 --branch 0.10.2 --single-branch https://github.com/eclipse-cyclonedds/cyclonedds.git
RUN cmake -B cyclonedds-build -S cyclonedds -DCMAKE_INSTALL_PREFIX=cyclonedds-install \
    && cmake --build cyclonedds-build --parallel 5 --target install

FROM cimg/rust:1.65.0
COPY --from=Builder /home/circleci/project/cyclonedds-install /usr/local/