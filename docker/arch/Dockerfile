FROM greyltc/archlinux
WORKDIR /build

# install tools and dependencies
RUN pacman -Syu --needed --noconfirm clang curl git pkg-config make udev libusb systemd-sysvcompat

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

# show backtraces
ENV RUST_BACKTRACE 1

# show tools
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v

# clone parity
RUN git clone https://github.com/paritytech/parity.git

# build parity
RUN cd parity && \
    cargo build --release --verbose && \
    ls /build/parity/target/release/parity && \
    strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/release/parity"]
