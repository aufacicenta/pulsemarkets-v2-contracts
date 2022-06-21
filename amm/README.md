# AMM & Prediction Markets

## What is a market

In simple terms, a market is an exchange of goods. Person A — the seller — has a product that person B — the buyer — wants. Person A sets a price — could be money or another good — for person B to buy an amount of such product.

## What are prediction markets

In Prediction Markets, the goods or products for sale are Outcomes of a Future Event. Think of a futbol soccer match where the outcomes are whether team A or team B will win. To create a market out of these outcomes, a market maker creates tokens (could be coins, cards, casino chips, etc.) that represents each outcome. The market maker who's providing a service puts a price on each outcome token, and can modify the price at will. Players or betters will then purchase a certain amount of these tokens before or during the event.

## What are Automated Market Makers, AMMs

Just as market makers create tokens for each outcome out of nothing, AMMs are machines that will also create tokens out of a set of outcomes. AMMs will also set and modify the price as it is programmed to do it.

## What are liquidity providers, LPs

When it comes to AMMs, there are certain terms that are not evident but are very important, such as Liquidity Providers. Simply put, LPs are buyers that will purchase an amount of an outcome token before the event even starts. Because there are involved risks such as the event not even happening, LPs would normally get a reward as an incentive to reduce that risk in the case the event does happen.

> Remember, LPs are simple buyers, in this sense, every buyer is also a LP, but LPs purchase tokens before the event starts because of the inherent risks.

## Prediction Markets' cycles

In Pulse, a prediction market has 5 cycles: market creation, market publishing, the presale, the sale and the resolution.

## The market creation

In the blockchain space, anyone can create a prediction market for any event. Also in the blockchain space a market is actually a smart contract. Creating this smart contract has the cost of pushing a program to a given blockchain, for Pulse it is the NEAR Protocol blockchain.

> Also in the blockchain space, nobody is the owner of the market contract; this means that no-one can steal the funds that buyers paid for the outcome tokens. Only the buyers can withdraw their funds and only the market resolutors or oracles can resolve a market. More about this later.

## Publishing the market

When the smart contract is ready, the next step is to publish it. This step creates the outcome tokens, their supply is set to `0` and their starting price is set by the formula: `1 / number_of_outcomes`.

> Since the smart contract is already created, anyone can publish the market. This allows your wealthy friend to publish it instead of you scratching your wallet.

> In Prediction Markets AMMs, an outcome token is actually a ledger that keeps the balance of each buyer. This ledger also allows to transfer the balance (if any) to another wallet.

In Pulse, when a market is published, each outcome has an associated Sputnik2 DAO proposal. More about this in the resolution cycle description.

A market is considered published when the outcome token — the ledger — for each outcome exists.

At this stage, the market is ready and the outcomes are for sale!

### The presale

When a prediction market is published, and before the event even happens, tokens are already for sale.

Before any purchase, the supply of each outcome token is set to `0` and it will be minted (the supply is incremented) by `amount * price` upon every purchase.

In Pulse, the presale is an exiting moment, because buyers get a bonus amount of tokens in exchange of their collateral token. This bonus is determined by `(market.published_at - market.starts_at) / env::block_timestamp() * (collateral_amount * outcome_token.price)`.

Also upon each purchase, the price of the purchased outcome token will be incremented by a `price_ratio`, a dynamic value that is calculated so that the price of each token is never greater than 1 and never lower than 0. The smart contract also makes sure that the price of the other outcome tokens is decremented so that the sum of all prices is always 1.

> In Pulse's PM AMM, the price is set to `1 / number_of_outcomes`. So the price of each soccer match AMM outcome token would start at `0.5`.

### The sale

When the presale period ends, meaning that the event has started and it is not yet finalized, the buyers no longer get a bonus for their purchase, in other words, they get 1:1 for their purchase at the current outcome token price.

#### Price dynamics

In prediction markets, the price of each outcome is always a value between 0 and 1. Also, the sum of all prices must always be 1.

### The resolution

Only when the event has ended, the market can be resoluted, meaning that someone or a group of people — commonly known as oracles — determine what was the winning outcome. When this happens, the price of the winning outcome is set to `1` and the other outcomes' prices is set to `0`. This will allow any outcome token holder to redeem their earnings by a proportional amount to the overall supply of such outcome token.

#### How are resolutions guaranteed?

In Pulse, not a single person can resolute a market, instead, a group of persons belonging to a Decentralized Autonomous Organization — DAO —, vote for the winning outcome within a Sputnik2 DAO.

> A Sputnik2 DAO is also a set of smart contracts that lets its members (wallets) vote on proposals. There's a special type of proposal called: FunctionCall proposal, that will call another smart contract function with a given set of parameters. In the case of Pulse's AMMs, the parameters determine what outcome to resolute.

## Deployment

To deploy this contract using Near CLI:

```
<!-- Create the account -->
near create-account amm-9.aufacicenta.testnet --masterAccount aufacicenta.testnet --initialBalance 10

<!-- Deploy the contract -->
near deploy --wasmFile target/wasm32-unknown-unknown/release/amm.wasm --accountId amm-9.aufacicenta.testnet --initFunction new --initArgs '{"market":{"description":"Mamifut vs. Cremas, Jul 1st, 2022","info":"market info","category":"sports","options":["mamifut","cremas"],"starts_at":1656698400000000000,"ends_at":1656703800000000000},"dao_account_id":"pulse-dao.sputnikv2.testnet","collateral_token_account_id":"usdt.fakes.testnet","fee_ratio":0.02,"resolution_window":259200000000000}'

<!-- Publish the outcomes -->
near call amm-9.aufacicenta.testnet publish --accountId aufacicenta.testnet --gas=60000000000000

<!-- Make a storage_deposit call to the NEP141 collateral -->
near call usdt.fakes.testnet storage_deposit --accountId amm-9.aufacicenta.testnet --deposit 0.00235

<!-- Transfer USDT (the collateral registered on deployment) to the AMM contract to Buy an outcome -->
near call usdt.fakes.testnet ft_transfer_call '{"receiver_id":"amm-9.aufacicenta.testnet","amount":"5000000","msg":"{\"BuyArgs\":{\"outcome_id\":0}}"}' --accountId aufacicenta.testnet --depositYocto 1 --gas=33000000000000

<!-- Get Outcome token (prices should have been updated) -->
near view amm-9.aufacicenta.testnet get_outcome_token '{"outcome_id":0}'
near view amm-9.aufacicenta.testnet get_outcome_token '{"outcome_id":1}'
```