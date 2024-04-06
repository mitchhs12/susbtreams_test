use crate::pb::erc20::types::v1::BalanceChanges;
use crate::utils::helper::{append_0x, get_erc20_token};
use substreams::scalar::BigInt;
use substreams::store::StoreGet;
use substreams::store::StoreGetString;
use substreams::errors::Error;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_entity_change::tables::Tables;

#[substreams::handlers::map]
pub fn graph_out(block: BalanceChanges, token: StoreGetString) -> Result<EntityChanges, Error> {
    let mut tables = Tables::new();

    for storage_change in block.balance_changes {
        let token_lookup = token.get_last(&storage_change.contract);
        let token_found = token_lookup.is_some();

        if token_found {
            let token_option = &get_erc20_token(&storage_change.contract);
            if token_option.is_some() {
                let token = token_option.clone().unwrap();

                tables
                    .create_row("Token", append_0x(&storage_change.contract))
                    .set("name", token.name.clone())
                    .set("decimals", token.decimals.clone())
                    .set("symbol", token.symbol.clone());
            }
        }

        let id = format!("{}:{}", storage_change.contract, storage_change.owner);

        if storage_change.change_type == 0 {
            continue;
        }

        tables.create_row("Account", append_0x(&storage_change.owner.clone()));

        tables
            .create_row("Balance", id)
            // contract address
            .set("token", append_0x(&storage_change.contract))
            // storage change
            .set("owner", append_0x(&storage_change.owner))
            .set(
                "balance",
                BigInt::try_from(storage_change.new_balance).unwrap_or(BigInt::zero()),
            );
    }

    Ok(tables.to_entity_changes())
}