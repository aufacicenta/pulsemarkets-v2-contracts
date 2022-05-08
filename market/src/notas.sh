
ID=poguz4.testnet
echo $ID


near delete mt1.$ID $ID
near create-account mt1.$ID --masterAccount $ID --initialBalance 10
near deploy --wasmFile src/market.wasm --accountId mt1.$ID
near call mt1.$ID new '{"market": {"description": "m1", "info": "i1", "category": "c1", "options": ["s1", "s2"], "expiration_date": 1651891079000000000, "resolution_window": 86400000000}, "dao_account_id": "ps1.sputnikv2.testnet"}' --accountId $ID

near view mt1.$ID get_data_data

near call mt1.$ID publish_market --accountId $ID --gas 60000000000000 --amount 0.3

near view mt1.$ID get_proposals

## Validar
# Sin opciones de mercado



#############
## Factory
#############

near delete fm14.$ID $ID
near create-account fm14.$ID --masterAccount $ID --initialBalance 10
near deploy --wasmFile res/market_factory.wasm --accountId fm14.$ID
near call fm14.$ID new --accountId $ID

near call fm14.$ID create_market '{"args": "eyJtYXJrZXQiOiB7ImRlc2NyaXB0aW9uIjogIm0xIiwgImluZm8iOiAiaTEiLCAiY2F0ZWdvcnkiOiAiYzEiLCAib3B0aW9ucyI6IFsiczEiLCAiczIiXSwgImV4cGlyYXRpb25fZGF0ZSI6IDE2NTE4OTEwNzkwMDAwMDAwMDAsICJyZXNvbHV0aW9uX3dpbmRvdyI6IDg2NDAwMDAwMDAwfSwgImRhb19hY2NvdW50X2lkIjogInBzMS5zcHV0bmlrdjIudGVzdG5ldCJ9", "num_options": 2}' --accountId $ID --gas 90000000000000 --amount 3.2

1643247480000000000
1651891079000000000 manana
86400000000 un dia
