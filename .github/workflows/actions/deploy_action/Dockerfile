FROM rust:bullseye

COPY deploy_script.sh deploy_script.sh

COPY . ./

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

RUN apt-get install -y npm
RUN npm install -g serverless
RUN chmod +x /deploy_script.sh
ENTRYPOINT ["/deploy_script.sh"]

