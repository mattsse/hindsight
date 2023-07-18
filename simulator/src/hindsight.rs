use crate::{
    config::Config,
    data::arbs::ArbDb,
    info,
    sim::processor::{simulate_backrun_arbs, H256Map},
    util::{get_ws_client, WsClient},
    Result,
};
use ethers::types::Transaction;
use futures::future;
use mev_share_sse::EventHistory;

#[derive(Clone, Debug)]
pub struct Hindsight {
    pub client: WsClient,
}

impl Hindsight {
    pub async fn new(config: Config) -> Result<Self> {
        let client = get_ws_client(Some(config.rpc_url_ws.to_owned())).await?;
        Ok(Self { client })
    }
    /// Process all transactions in `txs` taking `batch_size` at a time to run
    /// in parallel.
    ///
    /// Saves results into DB after each batch.
    pub async fn process_orderflow(
        self,
        txs: &Vec<Transaction>,
        batch_size: usize,
        connect: Option<Box<ArbDb>>,
        event_map: H256Map<EventHistory>,
    ) -> Result<()> {
        info!("loaded {} transactions total...", txs.len());
        let mut processed_txs = 0;
        while processed_txs < txs.len() {
            let mut handlers = vec![];
            let txs_batch = txs
                .iter()
                .skip(processed_txs)
                .take(batch_size)
                .map(|tx| tx.to_owned())
                .collect::<Vec<Transaction>>();
            processed_txs += txs_batch.len();
            info!("processing {} txs", txs_batch.len());
            for tx in txs_batch {
                let event_map = event_map.clone();
                let client = self.client.clone();
                handlers.push(tokio::spawn(async move {
                    simulate_backrun_arbs(&client, tx, &event_map).await.ok()
                }));
            }
            let results = future::join_all(handlers).await;
            let results = results
                .into_iter()
                .filter(|res| res.is_ok())
                .map(|res| res.unwrap())
                .filter(|res| res.is_some())
                .map(|res| res.unwrap())
                .collect::<Vec<_>>();
            info!("batch results: {:#?}", results);
            if let Some(db) = connect.to_owned() {
                // can't do && with a `let` in the conditional
                if !results.is_empty() {
                    db.to_owned().write_arbs(results).await?;
                }
            }
        }
        Ok(())
    }
}
