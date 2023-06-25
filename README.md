# token-listing-sniper-bot

rust based implementation token listing sninper bot.

Tested with

- Pancakeswap (Binance Smart Chain) ✔
- Uniswap (ethereum) ✔

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

## Development Environment

### Pre-Requisites

- pkg-config
- libssl-dev
