use anyhow::Result;
use tokio::net::{lookup_host, ToSocketAddrs};
use std::net::SocketAddr;
use std::collections::{BTreeMap, HashMap};

use crate::frame::response::Response;
use crate::frame::response::result;
use crate::query::Query;
use crate::prepared_statement::PreparedStatement;
use crate::transport::connection::Connection;
use crate::transport::Compression;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node {
    // TODO: a potentially node may have multiple addresses, remember them?
    // but we need an Ord instance on Node
    addr: SocketAddr,
}

pub struct Session {
    // invariant: nonempty
    pool: HashMap<Node, Connection>,
}

impl Session {
    pub async fn connect(addr: impl ToSocketAddrs + Clone, compression: Option<Compression>) -> Result<Self> {
        let mut options = HashMap::new();
        if let Some(compression) = &compression {
            let val = match compression {
                Compression::LZ4 => "lz4",
            };
            options.insert("COMPRESSION".to_string(), val.to_string());
        }

        let resolved = lookup_host(addr.clone()).await?.next().map_or(
            Err(anyhow!("no addresses found")), |a| Ok(a))?;
        let connection = Connection::new(addr, compression).await?;
        connection.startup(options).await?;

        let pool = vec![(Node{ addr: resolved }, connection)].into_iter().collect();

        Ok(Session { pool })
    }

    // TODO: Should return an iterator over results
    // actually, if we consider "INSERT" a query, then no.
    // But maybe "INSERT" and "SELECT" should go through different methods,
    // so we expect "SELECT" to always return Vec<result::Row>?
    pub async fn query(&self, query: impl Into<Query>) -> Result<Option<Vec<result::Row>>> {
        let result = self.any_connection().query(&query.into()).await?;
        match result {
            Response::Error(err) => {
                Err(err.into())
            }
            Response::Result(result::Result::Rows(rs)) => {
                Ok(Some(rs.rows))
            }
            Response::Result(_) => {
                Ok(None)
            }
            _ => Err(anyhow!("Unexpected frame received")),
        }
    }

    pub async fn prepare(&self, query: String) -> Result<PreparedStatement> {
        let result = self.any_connection().prepare(query.clone()).await?;
        match result {
            Response::Error(err) => {
                Err(err.into())
            }
            Response::Result(result::Result::Prepared(p)) => {
                Ok(PreparedStatement::new(p.id, query))
            }
            _ => return Err(anyhow!("Unexpected frame received")),
        }
    }

    fn any_connection(&self) -> &Connection {
        self.pool.values().next().unwrap()
    }
}