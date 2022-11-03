# Wrap

CosmWasm contract for wrap native coins into CW20.

## Build

```
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

## Optimize

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9
```
