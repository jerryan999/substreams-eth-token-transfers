mod abi;
mod pb;

use pb::sinkfiles::Lines;
use pb::transfers::transfer::Schema;
use pb::transfers::Transfer;
use pb::transfers::Transfers;

use substreams::log;
use substreams::scalar::BigInt;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

use abi::erc1155::events::TransferBatch as ERC1155TransferBatchEvent;
use abi::erc1155::events::TransferSingle as ERC1155TransferSingleEvent;
use abi::erc20::events::Transfer as ERC20TransferEvent;
use abi::erc721::events::Transfer as ERC721TransferEvent;

substreams_ethereum::init!();

/// Extracts transfers events from the contract(s)
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<Transfers, substreams::errors::Error> {
    Ok(Transfers {
        transfers: get_transfers(&blk).collect(),
    })
}

/// Extracts transfers events from the contract(s)
#[substreams::handlers::map] 
fn jsonl_out(blk: eth::Block) -> Result<Lines, substreams::errors::Error> {
    Ok(Lines {
        lines: get_transfers(&blk)
            .map(|trx| serde_json::to_string(&trx).unwrap())
            .collect(),
    })
}

/// Extracts transfers events from the filters contracts
#[substreams::handlers::map]  // means 
fn jsonl_out_with_filter(blk: eth::Block) -> Result<Lines, substreams::errors::Error> { 
    Ok(Lines {
        lines: get_transfers(&blk)
            .filter(|trx| {
                let contracts = vec![
                    "06a6a7af298129e3a2ab396c9c06f91d3c54aba8",
                    "3bf3d4e80b91d2731abcb154c21ad35abb417fd2",
                    "552d72f86f04098a4eaeda6d7b665ac12f846ad2",
                    "6966730b1435168880b35faa1e75de0988ee2e39",
                    "71c118b00759b0851785642541ceb0f4ceea0bd5",
                    "79986af15539de2db9a5086382daeda917a9cf0c",
                    "abc7e6c01237e8eef355bba2bf925a730b714d5f",
                    "c70be5b7c19529ef642d16c10dfe91c58b5c3bf0",
                    "d6076efe1e577deec21afab6ed383b47e9d8dec6",
                    "cc9a66acf8574141b0e025202dd57649765a4be7",

                    "847342e826d943e0c7950d0a88882b75a1b19b9f",
                    "ae36155a55f04a696b8362777620027882b31db5",
                    "7907f43283202b0a821fea95157fecf0c751dcb5",
                    "b5f1457d6fba1956fb8d31b0b7caca14bde0be4b",
                    "0167e9bcc26d02bf71113bbd1b6ea8c9549cd52c",
                    "15dd37a83564559927bbdcdbfcf85f60a2c6a419",
                    "048aa6e69d5117bd665af4128383a6982b8367b3",
                    "865ec58b06bf6305b886793aa20a2da31d034e68",
                    "f81421fc15300c5a8cca9afe12f5cbad502fa756",
                    "8b0fde007458ee153bd0f66cd448af5fb3d99b43",
                    "efcb0688c679d99ee1b78876e11ebe5be2e4292b",
                    "d084b83c305dafd76ae3e1b4e1f1fe2ecccb3988",
                    "22c5543d1a35178cb03b33f929a959145e538532",
                    "35a5cb585d51d836922b78a9bb1f5c04635c39b6",
                    "8dc89f4716e027394bba225b82328c1ea2ea58bf",
                    "f203ca1769ca8e9e8fe1da9d147db68b6c919817",
                    "08f40811c7d6c013744166f3d4cb1a9a92d3d54e",
                    "de4ce5447ce0c67920a1371605a39187cb6847c8",
                    "73350fdf1bfd08555d6fe440d3ffa87104d1da19",
                    "436da116249044e8b4464f0cf21dd93311d88190",
                ];
                contracts.contains(&trx.contract.as_str())
            })
            .map(|trx| serde_json::to_string(&trx).unwrap())
            .collect(),
    })
}


/// Extracts transfers events from the contract(s)
#[substreams::handlers::map]
fn csv_out(blk: eth::Block) -> Result<Lines, substreams::errors::Error> {
    Ok(Lines {
        lines: get_transfers(&blk).map(|trx| trx.to_csv()).collect(),
    })
}

fn get_transfers<'a>(blk: &'a eth::Block) -> impl Iterator<Item = Transfer> + 'a {
    blk.receipts().flat_map(|receipt| {
        let hash = &receipt.transaction.hash;

        receipt.receipt.logs.iter().flat_map(|log| {
            let contract = &log.address;
            if let Some(event) = ERC20TransferEvent::match_and_decode(log) {
                return vec![new_erc20_transfer(blk, contract, hash, log.block_index.to_string(), event)];
            }

            if let Some(event) = ERC721TransferEvent::match_and_decode(log) {
                return vec![new_erc721_transfer(blk, contract, hash, log.block_index.to_string(), event)];
            }

            if let Some(event) = ERC1155TransferSingleEvent::match_and_decode(log) {
                return vec![new_erc1155_single_transfer(blk, contract, hash, log.block_index.to_string(), event)];
            }

            if let Some(event) = ERC1155TransferBatchEvent::match_and_decode(log) {
                return new_erc1155_batch_transfer(blk,contract, hash, log.block_index.to_string(), event);
            }

            vec![]
        })
    })
}

fn new_erc20_transfer(blk:&eth::Block, contract: &[u8], hash: &[u8], log_index: String, event: ERC20TransferEvent) -> Transfer {
    let header = blk.header.as_ref().unwrap();
    let timestamp = header.timestamp.as_ref().unwrap().seconds as u64;

    Transfer {
        schema: schema_to_string(Schema::Erc20),
        from: Hex(&event.from).to_string(),
        to: Hex(&event.to).to_string(),
        quantity: event.value.to_string(),
        trx_hash: Hex(hash).to_string(),
        log_index: log_index.to_string(),
        operator: "".to_string(),
        token_id: "".to_string(),
        contract: Hex(contract).to_string(),
        block_number: blk.number,
        block_timestamp: timestamp}
}

fn new_erc721_transfer(blk:&eth::Block,contract:&[u8], hash: &[u8], log_index:  String, event: ERC721TransferEvent) -> Transfer {
    let header = blk.header.as_ref().unwrap();
    let timestamp = header.timestamp.as_ref().unwrap().seconds as u64;


    Transfer {
        schema: schema_to_string(Schema::Erc721),
        from: Hex(&event.from).to_string(),
        to: Hex(&event.to).to_string(),
        quantity: "1".to_string(),
        trx_hash: Hex(hash).to_string(),
        log_index: log_index.to_string(),
        token_id: event.token_id.to_string(),
        operator: "".to_string(),
        contract: Hex(contract).to_string(),
        block_number: blk.number,
        block_timestamp: timestamp
    }
}

fn new_erc1155_single_transfer(
    blk: &eth::Block,
    contract:&[u8],  
    hash: &[u8],
    log_index :String,
    event: ERC1155TransferSingleEvent,
) -> Transfer {
    new_erc1155_transfer(
        blk,
        contract,
        hash,
        log_index,
        &event.from,
        &event.to,
        &event.id,
        &event.value,
        &event.operator,
    )
}

fn new_erc1155_batch_transfer(
    blk: &eth::Block,
    contract:&[u8],
    hash: &[u8],
    log_index:  String,
    event: ERC1155TransferBatchEvent,
) -> Vec<Transfer> {
    if event.ids.len() != event.values.len() {
        log::info!("There is a different count for ids ({}) and values ({}) in transaction {} for log at block index {}, ERC1155 spec says lenght should match, ignoring the log completely for now",
            event.ids.len(),
            event.values.len(),
            Hex(&hash).to_string(),
            log_index,
        );

        return vec![];
    }

    event
        .ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let value = event.values.get(i).unwrap();

            new_erc1155_transfer(
                blk,
                contract,
                hash,
                log_index.clone() + "/" + &i.to_string(),   // This is userful for downstream processing
                &event.from,
                &event.to,
                id,
                value,
                &event.operator,
            )
        })
        .collect()
}

fn new_erc1155_transfer(
    blk: &eth::Block,
    contract:&[u8],
    hash: &[u8],
    log_index:  String,
    from: &[u8],
    to: &[u8],
    token_id: &BigInt,
    quantity: &BigInt,
    operator: &[u8],
) -> Transfer {
    let header = blk.header.as_ref().unwrap();
    let timestamp = header.timestamp.as_ref().unwrap().seconds as u64;

    Transfer {
        schema: schema_to_string(Schema::Erc1155),
        from: Hex(from).to_string(),
        to: Hex(to).to_string(),
        quantity: quantity.to_string(),
        trx_hash: Hex(hash).to_string(),
        log_index: log_index.to_string(),
        operator: Hex(operator).to_string(),
        token_id: token_id.to_string(),
        contract: Hex(contract).to_string(),
        block_number: blk.number,
        block_timestamp: timestamp
    }
}

fn schema_to_string(schema: Schema) -> String {
    match schema {
        Schema::Erc20 => "erc20",
        Schema::Erc721 => "erc721",
        Schema::Erc1155 => "erc1155",
    }
    .to_string()
}
