// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use core::{iter, num::NonZeroU32};

fn main() {
    // The `smoldot_light` library uses the `log` crate to emit logs.
    // We need to register some kind of logs listener, in this example `env_logger`.
    // See also <https://docs.rs/log>.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Now initialize the client. This does nothing except allocate resources.
    // The `Client` struct requires a generic parameter that provides platform bindings. In this
    // example, we provide `DefaultPlatform`, which are the "plug and play" default platform.
    // Any advance usage, such as embedding a client in WebAssembly, will likely require a custom
    // implementation of these bindings.
    let mut client =
        smoldot_light::Client::new(smoldot_light::platform::default::DefaultPlatform::new(
            env!("CARGO_PKG_NAME").into(),
            env!("CARGO_PKG_VERSION").into(),
        ));

    // Ask the client to connect to a chain.
    let smoldot_light::AddChainSuccess {
        chain_id,
        json_rpc_responses,
    } = client
        .add_chain(smoldot_light::AddChainConfig {
            // The most important field of the configuration is the chain specification. This is a
            // JSON document containing all the information necessary for the client to connect to said
            // chain.
            specification: include_str!("../../demo-chain-specs/polkadot.json"),

            // Configures some constants about the JSON-RPC endpoints.
            // It is also possible to pass `Disabled`, in which case the chain will not be able to
            // handle JSON-RPC requests. This can be used to save up some resources.
            json_rpc: smoldot_light::AddChainConfigJsonRpc::Enabled {
                // Maximum number of JSON-RPC in the queue of requests waiting to be processed.
                // This parameter is necessary for situations where the JSON-RPC clients aren't
                // trusted. If you control all the requests that are sent out and don't want them
                // to fail, feel free to pass `u32::max_value()`.
                max_pending_requests: NonZeroU32::new(128).unwrap(),
                // Maximum number of active subscriptions before new ones are automatically
                // rejected. Any JSON-RPC request that causes the server to generate notifications
                // counts as a subscription.
                // While a typical reasonable value would be for example 64, existing UIs tend to
                // start a lot of subscriptions, and a value such as 1024 is recommended.
                // Similarly, if you don't want any limit, feel free to pass `u32::max_value()`.
                max_subscriptions: 1024,
            },

            // This field is necessary only if adding a parachain.
            potential_relay_chains: iter::empty(),

            // After a chain has been added, it is possible to extract a "database" (in the form of a
            // simple string). This database can later be passed back the next time the same chain is
            // added again.
            // A database with an invalid format is simply ignored by the client.
            // In this example, we don't use this feature, and as such we simply pass an empty string,
            // which is intentionally an invalid database content.
            database_content: "",

            // The client gives the possibility to insert an opaque "user data" alongside each chain.
            // This avoids having to create a separate `HashMap<ChainId, ...>` in parallel of the
            // client.
            // In this example, this feature isn't used. The chain simply has `()`.
            user_data: (),
        })
        .unwrap();

    // The chain is now properly initialized.

    // `json_rpc_responses` can only be `None` if we had passed `json_rpc: Disabled` in the
    // configuration.
    let mut json_rpc_responses = json_rpc_responses.unwrap();

    // Send a JSON-RPC request to the chain.
    // The example here asks the client to send us notifications whenever the new best block has
    // changed.
    // Calling this function only queues the request. It is not processed immediately.
    // An `Err` is returned immediately if and only if the request isn't a proper JSON-RPC request
    // or if the channel of JSON-RPC responses is clogged.
    client
        .json_rpc_request(
            r#"{"id":1,"jsonrpc":"2.0","method":"chain_subscribeNewHeads","params":[]}"#,
            chain_id,
        )
        .unwrap();

    // Now block the execution forever and print the responses received on the channel of
    // JSON-RPC responses.
    smol::block_on(async move {
        loop {
            let response = json_rpc_responses.next().await.unwrap();
            println!("JSON-RPC response: {response}");
        }
    })
}
