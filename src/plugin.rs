use crate::{discord, hank};
use extism::{host_fn, UserData, ValType};
use hank_transport::{Message, PluginCommand, PluginMetadata, PluginResult};
use sea_orm::{Database, DatabaseConnection, FromQueryResult};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use twilight_model::id::Id;

#[derive(Clone, Debug)]
pub struct Plugin {
    /// The plugins metadata.
    pub metadata: PluginMetadata,

    database: Option<DatabaseConnection>,
    plugin_tx: mpsc::Sender<(PluginCommand, oneshot::Sender<PluginResult>)>,
}

host_fn!(send_message(message: Message) {
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        discord()
            .create_message(Id::new(message.channel_id.parse().unwrap()))
            .content(&message.content)
            .unwrap()
            .await
    });
    Ok(())
});

host_fn!(db_query(query: String) -> String {
    let ret = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move {
            hank().query_database("wordle".into(), query).await
        })
    });

    Ok(ret)
});

impl Plugin {
    pub async fn new(path: PathBuf) -> Self {
        let (plugin_tx, mut plugin_rx) =
            mpsc::channel::<(PluginCommand, oneshot::Sender<PluginResult>)>(32);

        tokio::spawn(async move {
            let manifest = extism::Manifest::new(vec![path]);
            let mut plugin = extism::PluginBuilder::new(manifest)
                .with_wasi(true)
                .with_function(
                    "send_message",
                    [ValType::I64],
                    [],
                    UserData::default(),
                    send_message,
                )
                .with_function(
                    "db_query",
                    [ValType::I64],
                    [ValType::I64],
                    UserData::default(),
                    db_query,
                )
                .build()
                .unwrap();

            while let Some((command, response)) = plugin_rx.recv().await {
                use PluginCommand::*;

                let result = if !plugin.function_exists(command.to_string()) {
                    PluginResult::FunctionNotFound
                } else {
                    let data: &[u8] = match command {
                        GetMetadata => plugin.call("get_metadata", "").unwrap(),
                        HandleMessage(message) => plugin
                            .call("handle_message", serde_json::to_string(&message).unwrap())
                            .unwrap(),
                    };

                    let data_str = String::from_utf8(data.to_vec()).unwrap();
                    serde_json::from_str::<PluginResult>(&data_str).unwrap()
                };

                response.send(result).unwrap();
            }
        });

        // Get the plugins metadata.
        let (resp_tx, resp_rx) = oneshot::channel();
        plugin_tx
            .send((PluginCommand::GetMetadata, resp_tx))
            .await
            .unwrap();
        let PluginResult::GetMetadata(metadata) = resp_rx.await.unwrap() else {
            panic!("Could not get plugin metadata");
        };

        let database = if metadata.database {
            let database_path = format!(
                "{}/db/{}.sqlite3",
                env!("CARGO_MANIFEST_DIR"),
                metadata.name
            );

            // Ensure database file exists.
            let _ = std::fs::File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(database_path.clone());

            let database = Database::connect(format!("sqlite://{}", database_path))
                .await
                .unwrap();

            Some(database)
        } else {
            None
        };

        Self {
            metadata,
            database,
            plugin_tx,
        }
    }

    pub async fn send_command(&self, cmd: PluginCommand) -> PluginResult {
        let (resp_tx, resp_rx) = oneshot::channel();

        self.plugin_tx.send((cmd, resp_tx)).await.unwrap();
        resp_rx.await.unwrap()
    }

    pub async fn query_database(&self, query: String) -> String {
        let database = self.database.as_ref().unwrap();

        let res: Vec<sea_orm::JsonValue> = sea_orm::JsonValue::find_by_statement(
            sea_orm::Statement::from_string(sea_orm::DbBackend::Sqlite, query),
        )
        .all(database)
        .await
        .unwrap();

        res.first()
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .to_string()
    }
}
