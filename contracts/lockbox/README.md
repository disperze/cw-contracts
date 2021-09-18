# Lockbox

A juno smartcontract for lock funds until expire. This contract allows native/ibc coins and CW20 tokens

## Instantiate
To create it, you must pass a max_lock_time param.

```rust
pub struct InstantiateMsg {
  pub max_lock_time: u64,
}
```

## Execute

| Msg     |  Description |   
|---------|--------------|
|`Lock {id: String, expire: Timestamp}`| Lock funds until expire timestamp |
|`IncreaseLock { id: String }`| Increase existing lock |
|`Unlock { id: String }`| Unlock funds|

## Query
| Msg     |  Description |   
|---------|--------------|
|`Lock { address: String, id: String }`| Get lock info by id |
|`AllLocks { address: String }`| Get all locks by user address |

## Example

Instantiate msg:
```json
{
  "max_lock_time": 157680000 
}
```
> max time: 5 years

Execut msg:
- Native coin (lock sent funds)
```json
{
  "lock": {
    "id": "1",
    "expire": "1627256261000000000"
  }
}
```
- CW20 tokens
```json
{
  "send": {
    "contract": "juno150993pnauf335xtp8pnu2jtlt87h2lsage4m29",
    "amount": "1000000",
    "msg": btoa('{"id": "1", "expire": "1627256261000000000"}') // lockbox msg in base64
}
```
> execute in cw20 contract
