FROM cimg/base:2023.09 AS Builder
RUN git clone --depth 1 --branch 0.10.2 --single-branch https://github.com/eclipse-cyclonedds/cyclonedds.git
RUN cmake -B cyclonedds-build -S cyclonedds -DCMAKE_INSTALL_PREFIX=cyclonedds-install \
    && cmake --build cyclonedds-build --parallel 5 --target install


FROM cimg/base:2023.09 AS FastDdsBuilder
USER root
RUN apt-get update && apt-get install -y libasio-dev libtinyxml2-dev

RUN git clone --depth 1 --branch v1.3.1 --single-branch https://github.com/eProsima/foonathan_memory_vendor.git
RUN cmake -B foonathan_memory_vendor-build -S foonathan_memory_vendor -DCMAKE_INSTALL_PREFIX=Fast-DDS-install -DBUILD_SHARED_LIBS=ON \
    && cmake --build foonathan_memory_vendor-build --parallel 5 --target install

RUN git clone --depth 1 --branch v1.1.1 --single-branch https://github.com/eProsima/Fast-CDR.git
RUN cmake -B Fast-CDR-build -S Fast-CDR -DCMAKE_INSTALL_PREFIX=Fast-DDS-install \
    && cmake --build Fast-CDR-build --parallel 5 --target install

RUN git clone --depth 1 --branch v2.10.2 --single-branch https://github.com/eProsima/Fast-DDS.git
RUN cmake -B Fast-DDS-build -S Fast-DDS -DCMAKE_INSTALL_PREFIX=Fast-DDS-install \
    && cmake --build Fast-DDS-build --parallel 5 --target install

FROM cimg/openjdk:11.0 AS FastDdsGenBuilder
RUN git clone --depth 1 --branch v3.0.0 --single-branch https://github.com/eProsima/Fast-DDS-Gen.git
RUN cd Fast-DDS-Gen && ./gradlew assemble


FROM cimg/rust:1.65.0
COPY --from=Builder /home/circleci/project/cyclonedds-install /usr/local/
COPY --from=FastDdsBuilder /home/circleci/project/Fast-DDS-install /usr/local/
COPY --from=FastDdsGenBuilder /home/circleci/project/Fast-DDS-Gen/share /usr/local/share