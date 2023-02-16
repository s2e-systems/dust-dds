FROM cimg/rust:1.62.0
RUN git clone --depth=1 https://github.com/eclipse-cyclonedds/cyclonedds.git \
    && mkdir cyclonedds-build
RUN cd cyclonedds-build \
    && cmake ../cyclonedds \
    && cmake --build . --parallel 5 \
    && sudo cmake --build . --target install
RUN rm -rf cyclonedds \
    && rm -rf cyclonedds-build