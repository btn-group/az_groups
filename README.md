<!-- PROJECT LOGO -->
# AZ Groups

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#setting-up-locally">Setting up locally</a></li>
      </ul>
    </li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

<p align="right">(<a href="#top">back to top</a>)</p>

### Built With

* [Cargo](https://doc.rust-lang.org/cargo/)
* [Rust](https://www.rust-lang.org/)
* [ink!](https://use.ink/)
* [OpenBrush](https://openbrush.io/)
* [Cargo Contract v2.0.1](https://github.com/paritytech/cargo-contract)
```zsh
cargo install --force --locked cargo-contract --version 2.0.1
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

To get a local copy up and running follow these simple example steps.

### Prerequisites

* A pre-requisite for compiling smart contracts is to have a stable Rust version and Cargo installed. Here's an [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).
* The first tool we will be installing is [cargo-contract](https://github.com/paritytech/cargo-contract), a CLI tool for helping setting up and managing WebAssembly smart contracts written with ink!.

### Checking code

```zsh
cargo checkmate
cargo sort
```

### Building contract

By default, cargo-contract builds the contract in debug mode. This means that the contract will e.g. print statements like

```sh
ink::env::debug_println!("magic number: {}", value);
```
to the node's console if debugging was enabled on the node ([instructions here](https://use.ink/faq#how-do-i-print-something-to-the-console-from-the-runtime)). To support functionality like this the debug build of a contract includes some heavy-weight logic.

For contracts that are supposed to run in production you should always build the contract with --release:
```sh
cargo contract build --release
```
This will ensure that nothing unnecessary is compiled into the Wasm blob, making your contract faster and cheaper to deploy and execute.

### Setting up locally

The [substrate-contracts-node](https://github.com/paritytech/substrate-contracts-node) is a simple Substrate blockchain which is configured to include the Substrate module for smart contract functionality – the contracts pallet (see [How it Works](https://use.ink/how-it-works) for more). It's a comfortable option if you want to get a quickstart. Download the binary [here](https://github.com/paritytech/substrate-contracts-node/releases).

[After successfully installing substrate-contracts-node](https://use.ink/getting-started/setup#installing-the-substrate-smart-contracts-node), you can start a local development chain by running:

```sh
substrate-contracts-node
```

You can interact with your node using the [Contracts UI](https://contracts-ui.substrate.io/). Once you have the webpage open, click on the dropdown selector at the top left corner and choose "Local Node".

Note that blocks are only created when you execute a function in substrate-contracts-node, so trigger a another function first if a function depends on a time delay.

## References 

- https://substrate.stackexchange.com/questions/3765/contract-storage-needs-nested-orderbooks-best-practice-way-to-structure-dapp/3993
