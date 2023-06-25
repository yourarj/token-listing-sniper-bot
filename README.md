# token-listing-sniper-bot

rust based implementation token listing sninper bot.

Tested with

- Pancakeswap **v1** (Binance Smart Chain) ✔
- Uniswap (ethereum) ✔

Not tested with
- Pancakeswap **v3** [*Smart Router*] (Binance Smart Chain) ⚠️


May work with

- Pangolin (Avalanche) ❔

## How to run bot

```
$block-bot --help

tool to sniff the liquidity add event for desired token buy them automatically

Usage: block-bot [OPTIONS] --wss <WSS> --contract <CONTRACT> --native <NATIVE> --token <TOKEN>

Options:
  -w, --wss <WSS>            wss provider
  -h, --http <HTTP>          http provider
  -c, --contract <CONTRACT>  exchange contract address to watch for liquidity add eve
      --native <NATIVE>      native token address. It'll be spent for buying
      --token <TOKEN>        token address. This token will be bought
  -h, --help                 Print help
  -V, --version              Print version
```

## e.g.
```
block-bot \
--wss wss://testnet-dex.binance.org/api/ \
--http https://data-seed-prebsc-1-s1.binance.org:8545 \
--contract 0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73 \
--native 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c \
--token 0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82
```

## Development Environment

### Pre-Requisites

- pkg-config
- libssl-dev
