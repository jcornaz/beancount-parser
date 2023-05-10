FROM gitpod/workspace-rust
RUN rustup update
RUN cargo install just cargo-hack cargo-watch cargo-msrv cargo-deny
