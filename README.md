# SOLSQL

A Solana program that enables SQL-like access to the Solana blockchain, supporting full CRUD operations.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)]()

## Table of Contents

- [Usage](#usage)
- [Features](#features)
- [License](#license)

## Usage
To interact with SOLSQL, you must use the **TypeScript client kit**. The kit is maintained in a separate repository:

[SOLSQL Kit Repository](https://github.com/AkliluYirgalem/solsql-kit)

Example usage with the kit:

```ts
import { SolSQLClient } from "solsql-kit";

async function main() {
    const solsql = new SolSQLClient("devnet");

    await solsql.createTable({
        authority: "./YOUR-KEYPAIR-FILE.json",
        tableName: "NewUsersTable",
        columns: {
            fname: {},
            lname: {},
            email: { unique: true },
        }
    });
}
```
## Features
- 💾 **Dynamic Data Storage** — supports flexible, resizable on-chain data structures.  
- 🧰 **TypeScript SDK** — simple, modern client library for interacting with the program.  
- 📦 **Lightweight Deployment** — minimal compute usage and rent cost.

## License
This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.
