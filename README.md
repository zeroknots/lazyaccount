## EIP-4337 Lazy Account

This is a simple implementation of EIP-4337 Lazy Account.

### Usage

Run docker compose to start local `anvil`, mock paymaster and ERC-4337 bundler (`alto`).

```bash
docker compose up
```

#### Execute

In a json file, provide the list of calls to be executed in the following format:

```json
{
    "entrypoint": "0x...",
    "executions": [
        {
            "target": "0x...",
            "value": "0",
            "callData": "0x..."
        },
        // ...
    ]
}
```

Then run the `execute` subcommand:

```bash
cargo run execute --account <smart-account-address> --input <path-to-json-file> --validator <validator-module-address>
```

Optionally, you can provide the custom public client and bundler endpoints:

```bash
cargo run execute --account <smart-account-address> --input <path-to-json-file> --validator <validator-module-address> --client <public-client-url> --bundler <bundler-url>
```

By default, it uses local `anvil` and `alto` instances at `8545` and `8546` ports respectively.

#### Install Module

To install a module, run the `module install` subcommand:

```bash
cargo run module install --account <address> --validator <validator-address> --module <module-address> --module-type-id <1|2|3|4> --data <hex-data-no-prefix>
```

Where module data is the address of the smart account that will be used to install the module.

#### Is Module Installed

To check if a module is installed, run the `module installed` subcommand:

```bash
cargo run module is-installed --account <address> --validator <validator-address> --module <module-address> --module-type-id <1|2|3|4> --data <hex-data-no-prefix>
```
