FROM cimg/rust:1.61.0
RUN git clone https://github.com/eclipse-cyclonedds/cyclonedds.git \
    && mkdir cyclonedds-build
WORKDIR cyclonedds-build
RUN cmake ../cyclonedds \
    && cmake --build . \
    && sudo cmake --build . --target install