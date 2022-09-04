Free Storage FT Contract
======

- Basic smart contract structure
- Jest sim tests
- Build local/docker binaries
- Build & deploy scripts

HOW TO INIT
======

```
export FT_ID=dev-1661900932898-58281346290206

near call $FT_ID new '{ 
    "operator_id": "'$FT_ID'", 
    "total_supply": "10000000000000000000000000000000000",
    "metadata": {
        "spec": "ft-1.0.0",
        "name": "TipToken",
        "symbol": "TT",
        "decimals": 18
    }
}' --accountId $FT_ID

near view $FT_ID ft_metadata '{}'
```


BUILD DOCKER ON M1:
===
Prepare docker
```
 clone https://github.com/near/near-sdk-rs/pull/720/files
 ./build_docker_m1.sh
```

Run docker buildx `contract-builder`
``` 
 ./build_docker_m1.sh
```

