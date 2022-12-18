# Token Listing Sniper Bot

An efficient bot written for trading token as soon as the liquidity for the pair is added to exchange. Exchanges supported(tested against) are uniswap, pancakeswap.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

### Added

- Integrate [clap](https://github.com/clap-rs/clap) 4.X
- FEATURE: Add bot **PAUSE** support. In paused mode bot will not watch liquidity add events.
- FEATURE: Add bot **WATCH** mode. In this mode bot will watch for liquidity add events but will not respond to them.

## [1.0.0] - 2022-12-18

### CHANGED

- Upgrade to stable [ethers](https://github.com/gakonst/ethers-rs) 1.X

## [0.1.0] - 2022-02-19

### Added

- Initial release of bot.
