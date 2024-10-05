export RUSTFLAGS="-L/opt/openssl/lib64/ -lcrypto -lssl -A unused"
export CPATH=/opt/homebrew/Cellar/musl-cross/0.9.9_1/libexec/x86_64-linux-musl/include
export OPENSSL_DIR=/opt/openssl/
export PKG_CONFIG_ALLOW_CROSS=1
export OPENSSL_STATIC=1
export CPPFLAGS=-I/opt/openssl/include
export LDFLAGS=-L/opt/openssl/lib64/

CROSS_COMPILE=x86_64-linux-musl- cargo build --release --target x86_64-unknown-linux-musl

export RUSTFLAGS="-A unused"
export RUSTFLAGS=$RUSTFLAGS' -C target-feature=+simd128'

./bin/mosquitto -c conf/mosquitto.conf -v

bin//mosquitto_pub -h localhost -p 3881 -t '/metapower/text/done/FDCBDEA1-BDC7-4443-B201-9D87B3FC4C65' -i 'MetaPowerAssistantAgent'  -m 'chat/download/'

./arduino-fwuploader firmware flash -i ../NINA_W102-v1.5.0-Nano-RP2040-Connect.bin  -b arduino:samd:nano_33_iot -a /dev/cu.usbmodem2101

dfx deploy matrix --network ic
dfx deploy agent --network ic
dfx cycles balance --network ic
dfx ledger balance --network ic
dfx canister logs matrix --network ic
dfx identity list
dfx identity get-principal
dfx cycles convert --amount 1.29 --network ic
dfx canister create agent --network ic
dfx identity whoami --network ic
dfx ledger account-id --network ic
dfx identity use metapowermatrix
dfx identity new metapowermatrix
dfx canister call agent hi
dfx canister call matrix initialize 'architecture'
candid-extractor target/wasm32-unknown-unknown/release/agent_canister_opt.wasm > src/agent/agent_canister.did
candid-extractor target/wasm32-unknown-unknown/release/matrix_canister_opt.wasm > src/matrix/matrix_canister.did
candid-extractor target/wasm32-unknown-unknown/release/pato_canister_opt.wasm > src/battery/pato_canister.did
