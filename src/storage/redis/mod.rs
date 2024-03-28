use {
    crate::storage::{deserialize, serialize, KeyValueStorage, StorageError, StorageResult},
    async_trait::async_trait,
    deadpool_redis::{
        redis::{AsyncCommands, Value},
        Config,
        Pool,
    },
    serde::{de::DeserializeOwned, Serialize},
    std::{fmt::Debug, time::Duration},
};

const LOCAL_REDIS_ADDR: &str = "redis://localhost:6379/0";

#[derive(Debug, Clone)]
pub enum Addr<'a> {
    Combined(&'a str),
    Separate { read: &'a str, write: &'a str },
}

impl<'a> Default for Addr<'a> {
    fn default() -> Self {
        Self::Combined(LOCAL_REDIS_ADDR)
    }
}

impl<'a> Addr<'a> {
    pub fn read(&self) -> &str {
        match self {
            Self::Combined(addr) => addr,
            Self::Separate { read, .. } => read,
        }
    }

    pub fn write(&self) -> &str {
        match self {
            Self::Combined(addr) => addr,
            Self::Separate { write, .. } => write,
        }
    }
}

impl<'a> From<(&'a Option<String>, &'a Option<String>)> for Addr<'a> {
    fn from(val: (&'a Option<String>, &'a Option<String>)) -> Self {
        match val {
            (Some(read), Some(write)) => Self::Separate { read, write },
            (Some(addr), None) => Self::Combined(addr),
            (None, Some(addr)) => Self::Combined(addr),
            _ => Default::default(),
        }
    }
}

/// A interface to interact with Redis cache.
#[derive(Clone)]
pub struct Redis {
    read_pool: Pool,
    write_pool: Pool,
}

impl Debug for Redis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Redis").finish()
    }
}

impl Redis {
    /// Instantiate a new Redis.
    pub fn new(addr: &Addr<'_>, pool_size: usize) -> StorageResult<Self> {
        let get_pool = |cfg: Config| -> Result<_, StorageError> {
            let pool = cfg
                .builder()
                .map_err(|e| StorageError::Other(format!("{e}")))?
                .max_size(pool_size)
                .build()
                .map_err(|e| StorageError::Other(format!("{e}")))?;

            Ok(pool)
        };

        let read_config = Config::from_url(addr.read());
        let read_pool = get_pool(read_config)?;

        let write_config = Config::from_url(addr.write());
        let write_pool = get_pool(write_config)?;

        Ok(Self {
            read_pool,
            write_pool,
        })
    }

    async fn set_internal(
        &self,
        key: &str,
        data: &[u8],
        ttl: Option<Duration>,
    ) -> StorageResult<()> {
        let mut conn = self
            .write_pool
            .get()
            .await
            .map_err(|e| StorageError::Connection(format!("{e}")))?;

        let res_fut = if let Some(ttl) = ttl {
            let ttl = ttl.as_secs();

            conn.set_ex(key, data, ttl)
        } else {
            conn.set(key, data)
        };

        res_fut
            .await
            .map_err(|e| StorageError::Other(format!("{e}")))?;

        Ok(())
    }
}

#[async_trait]
impl<T> KeyValueStorage<T> for Redis
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    async fn get(&self, key: &str) -> StorageResult<Option<T>> {
        self.read_pool
            .get()
            .await
            .map_err(|e| StorageError::Connection(format!("{e}")))?
            .get::<_, Value>(key)
            .await
            .map_err(|e| StorageError::Other(format!("{e}")))
            .map(|data| match data {
                Value::Nil => Ok(None),
                Value::Data(data) => Ok(Some(deserialize(&data)?)),
                _ => Err(StorageError::Deserialize),
            })?
    }

    async fn set(&self, key: &str, value: &T, ttl: Option<Duration>) -> StorageResult<()> {
        let data = serialize(value)?;
        self.set_internal(key, &data, ttl).await
    }

    async fn set_serialized(
        &self,
        key: &str,
        data: &[u8],
        ttl: Option<Duration>,
    ) -> StorageResult<()> {
        self.set_internal(key, data, ttl).await
    }

    async fn del(&self, key: &str) -> StorageResult<()> {
        self.write_pool
            .get()
            .await
            .map_err(|e| StorageError::Connection(format!("{e}")))?
            .del(key)
            .await
            .map_err(|e| StorageError::Other(format!("{e}")))
    }
}
