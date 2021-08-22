# Split smartcontract

Cosmwasm contract that lets you route funds continuously to a number of addresses.

## Instantiation

To create it, you must pass in a list of users and receipted coin.

```rust
pub struct InstantiateMsg {
    pub native_coin: String,
    pub users: Vec<User>,
}

pub struct User {
    pub address: Addr,
    // percentage allocation
    pub percent: Decimal,
}
```

Any user can send coins to contract address from any wallet (ex: Keplr).

## Execute

| Msg     |  Description |   
|---------|--------------|
|`Split{}`| User claims his percentage allocation|


## Example

Instantiate msg:
```json
{
  "native_coin": "ujuno",
  "users": [
    {
      "address": "juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn",
      "percent": "0.50"
    },
    {
      "address": "juno1q4aw0vtcydtn7lqmkfprm4ncmr4jdj70ggc6sp",
      "percent": "0.30"
    },
    {
      "address": "juno1qesady6wm5v7yh497xgq6uu8hnayr03u8ucrud",
      "percent": "0.20"
    }
  ]
}
```

Execute Msg:
```json
{
  "split": {}
}
```