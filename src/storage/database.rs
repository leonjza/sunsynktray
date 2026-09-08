use crate::domain::{EnergySnapshot, HistoryPoint, HistorySeries};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc, Mutex, OnceLock,
};

static DATABASE_INSTANCE: OnceLock<Mutex<Option<Arc<Database>>>> = OnceLock::new();

enum Command {
    SaveBackfillState {
        email: String,
        serial: String,
        target_date: String,
        oldest_date: String,
        status: String,
        last_error: Option<String>,
    },
    SaveSnapshot {
        email: String,
        snapshot: EnergySnapshot,
    },
    SaveHistory {
        email: String,
        serial: String,
        date: String,
        history: Vec<HistorySeries>,
    },
    SaveHistoryAndBackfill {
        email: String,
        serial: String,
        date: String,
        history: Vec<HistorySeries>,
        next_date: String,
        oldest_date: String,
        status: String,
        cancel_epoch: Arc<AtomicU64>,
        expected_epoch: u64,
        reply: mpsc::Sender<Result<()>>,
    },
    Flush {
        reply: mpsc::Sender<()>,
    },
    Shutdown,
}

enum ReadCommand {
    LoadBackfillState {
        email: String,
        serial: String,
        reply: mpsc::Sender<Result<Option<BackfillState>>>,
    },
    LoadSnapshot {
        email: String,
        reply: mpsc::Sender<Result<Option<EnergySnapshot>>>,
    },
    LoadHistory {
        email: String,
        serial: Option<String>,
        date: String,
        reply: mpsc::Sender<Result<Vec<HistorySeries>>>,
    },
    Shutdown,
}

#[derive(Clone, Debug)]
pub(crate) struct BackfillState {
    pub(crate) target_date: String,
    pub(crate) status: String,
    pub(crate) last_error: Option<String>,
}

pub(crate) struct Database {
    sender: mpsc::Sender<Command>,
    read_sender: mpsc::Sender<ReadCommand>,
    last_snapshots: Arc<Mutex<HashMap<String, EnergySnapshot>>>,
}

impl Database {
    pub(crate) fn open() -> Result<Arc<Self>> {
        let instance = DATABASE_INSTANCE.get_or_init(|| Mutex::new(None));
        let mut instance = instance.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(database) = instance.as_ref() {
            return Ok(database.clone());
        }
        let directory = crate::platform::app_data_dir()?;
        std::fs::create_dir_all(&directory).with_context(|| {
            format!(
                "could not create database directory {}",
                directory.display()
            )
        })?;
        let path = directory.join("suntray.sqlite3");
        let (sender, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
        let (read_sender, read_receiver) = mpsc::channel();
        let (read_ready_sender, read_ready_receiver) = mpsc::sync_channel(0);
        let last_snapshots = Arc::new(Mutex::new(HashMap::new()));
        let writer_last_snapshots = last_snapshots.clone();
        let writer_path = path.clone();
        std::thread::Builder::new()
            .name("suntray-database".into())
            .spawn(move || {
                let result = Connection::open(&writer_path).and_then(|connection| {
                    configure(&connection)?;
                    ready_sender.send(Ok(())).ok();
                    run(connection, receiver, writer_last_snapshots);
                    Ok(())
                });
                if let Err(error) = result {
                    let _ = ready_sender.send(Err(error));
                }
            })
            .context("could not start database worker")?;
        ready_receiver.recv().context("database worker stopped")??;
        std::thread::Builder::new()
            .name("suntray-database-read".into())
            .spawn(move || {
                let result = Connection::open(&path).and_then(|connection| {
                    configure(&connection)?;
                    read_ready_sender.send(Ok(())).ok();
                    read(connection, read_receiver);
                    Ok(())
                });
                if let Err(error) = result {
                    let _ = read_ready_sender.send(Err(error));
                }
            })
            .context("could not start database read worker")?;
        read_ready_receiver
            .recv()
            .context("database read worker stopped")??;
        let database = Arc::new(Self {
            sender,
            read_sender,
            last_snapshots,
        });
        *instance = Some(database.clone());
        Ok(database)
    }

    pub(crate) fn load_snapshot(&self, email: String) -> Result<Option<EnergySnapshot>> {
        let (reply, result) = mpsc::channel();
        self.read_sender
            .send(ReadCommand::LoadSnapshot { email, reply })?;
        result.recv().context("database worker stopped")?
    }

    pub(crate) fn load_history(
        &self,
        email: String,
        serial: Option<String>,
        date: String,
    ) -> Result<Vec<HistorySeries>> {
        let (reply, result) = mpsc::channel();
        self.read_sender.send(ReadCommand::LoadHistory {
            email,
            serial,
            date,
            reply,
        })?;
        result.recv().context("database worker stopped")?
    }

    pub(crate) fn save_snapshot(&self, email: String, snapshot: EnergySnapshot) {
        let mut last_snapshots = self
            .last_snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if last_snapshots.get(&email) == Some(&snapshot) {
            return;
        }
        if self
            .sender
            .send(Command::SaveSnapshot {
                email: email.clone(),
                snapshot: snapshot.clone(),
            })
            .is_ok()
        {
            last_snapshots.insert(email, snapshot);
        }
    }

    pub(crate) fn load_backfill_state(
        &self,
        email: String,
        serial: String,
    ) -> Result<Option<BackfillState>> {
        let (reply, result) = mpsc::channel();
        self.read_sender.send(ReadCommand::LoadBackfillState {
            email,
            serial,
            reply,
        })?;
        result.recv().context("database worker stopped")?
    }

    pub(crate) fn save_backfill_state(
        &self,
        email: String,
        serial: String,
        target_date: String,
        oldest_date: String,
        status: String,
        last_error: Option<String>,
    ) {
        let _ = self.sender.send(Command::SaveBackfillState {
            email,
            serial,
            target_date,
            oldest_date,
            status,
            last_error,
        });
    }

    pub(crate) fn save_history(
        &self,
        email: String,
        serial: String,
        date: String,
        history: Vec<HistorySeries>,
    ) {
        let _ = self.sender.send(Command::SaveHistory {
            email,
            serial,
            date,
            history,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn save_history_and_backfill(
        &self,
        email: String,
        serial: String,
        date: String,
        history: Vec<HistorySeries>,
        next_date: String,
        oldest_date: String,
        status: String,
        cancel_epoch: Arc<AtomicU64>,
        expected_epoch: u64,
    ) -> Result<()> {
        let (reply, result) = mpsc::channel();
        self.sender.send(Command::SaveHistoryAndBackfill {
            email,
            serial,
            date,
            history,
            next_date,
            oldest_date,
            status,
            cancel_epoch,
            expected_epoch,
            reply,
        })?;
        result.recv().context("database worker stopped")?
    }

    pub(crate) fn flush(&self) {
        let (reply, result) = mpsc::channel();
        if self.sender.send(Command::Flush { reply }).is_ok() {
            let _ = result.recv();
        }
    }

    pub(crate) fn shutdown(&self) {
        let _ = self.sender.send(Command::Shutdown);
        let _ = self.read_sender.send(ReadCommand::Shutdown);
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        let _ = self.read_sender.send(ReadCommand::Shutdown);
    }
}

fn configure(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS accounts (id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE);
         CREATE TABLE IF NOT EXISTS latest_snapshots (
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            serial TEXT NOT NULL, observed_at TEXT NOT NULL, pv_watts REAL NOT NULL,
            load_watts REAL NOT NULL, grid_watts REAL NOT NULL, battery_watts REAL NOT NULL,
            battery_soc REAL NOT NULL, solar_yield_kwh REAL, updated_at TEXT,
            pv_to INTEGER, to_load INTEGER, to_grid INTEGER, to_battery INTEGER,
            battery_to INTEGER, grid_to INTEGER, PRIMARY KEY(account_id, serial));
         CREATE TABLE IF NOT EXISTS history_points (
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            serial TEXT NOT NULL, point_time TEXT NOT NULL, label TEXT NOT NULL,
            watts REAL NOT NULL, source_date TEXT NOT NULL, fetched_at TEXT NOT NULL,
            PRIMARY KEY(account_id, serial, source_date, point_time, label));
         CREATE INDEX IF NOT EXISTS history_lookup ON history_points(account_id, serial, source_date, point_time);
         CREATE TABLE IF NOT EXISTS backfill_state (
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            serial TEXT NOT NULL, target_date TEXT NOT NULL, oldest_date TEXT,
            status TEXT NOT NULL DEFAULT 'pending', last_error TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY(account_id, serial));",
    )
}

fn run(
    connection: Connection,
    receiver: mpsc::Receiver<Command>,
    last_snapshots: Arc<Mutex<HashMap<String, EnergySnapshot>>>,
) {
    for command in receiver {
        let stop = matches!(&command, Command::Shutdown);
        match command {
            Command::SaveBackfillState {
                email,
                serial,
                target_date,
                oldest_date,
                status,
                last_error,
            } => {
                if let Err(error) = save_backfill_state(
                    &connection,
                    &email,
                    &serial,
                    &target_date,
                    &oldest_date,
                    &status,
                    last_error.as_deref(),
                ) {
                    tracing::warn!(%error, "could not save SQLite backfill state");
                }
            }
            Command::SaveSnapshot { email, snapshot } => {
                if let Err(error) = save_snapshot(&connection, &email, &snapshot) {
                    let mut cached = last_snapshots
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if cached.get(&email) == Some(&snapshot) {
                        cached.remove(&email);
                    }
                    tracing::warn!(%error, "could not save SQLite snapshot");
                }
            }
            Command::SaveHistory {
                email,
                serial,
                date,
                history,
            } => {
                if let Err(error) = save_history(&connection, &email, &serial, &date, &history) {
                    tracing::warn!(%error, "could not save SQLite history");
                }
            }
            Command::SaveHistoryAndBackfill {
                email,
                serial,
                date,
                history,
                next_date,
                oldest_date,
                status,
                cancel_epoch,
                expected_epoch,
                reply,
            } => {
                let result = save_history_and_backfill(
                    &connection,
                    &email,
                    &serial,
                    &date,
                    &history,
                    &next_date,
                    &oldest_date,
                    &status,
                    &cancel_epoch,
                    expected_epoch,
                );
                let _ = reply.send(result.map_err(Into::into));
            }
            Command::Flush { reply } => {
                let _ = reply.send(());
            }
            Command::Shutdown => {}
        }
        if stop {
            break;
        }
    }
}

fn read(connection: Connection, receiver: mpsc::Receiver<ReadCommand>) {
    for command in receiver {
        let stop = matches!(&command, ReadCommand::Shutdown);
        match command {
            ReadCommand::LoadBackfillState {
                email,
                serial,
                reply,
            } => {
                let _ = reply
                    .send(load_backfill_state(&connection, &email, &serial).map_err(Into::into));
            }
            ReadCommand::LoadSnapshot { email, reply } => {
                let _ = reply.send(load_snapshot(&connection, &email).map_err(Into::into));
            }
            ReadCommand::LoadHistory {
                email,
                serial,
                date,
                reply,
            } => {
                let _ = reply.send(
                    load_history(&connection, &email, serial.as_deref(), &date).map_err(Into::into),
                );
            }
            ReadCommand::Shutdown => {}
        }
        if stop {
            break;
        }
    }
}

fn account_id(connection: &Connection, email: &str) -> rusqlite::Result<i64> {
    connection.execute(
        "INSERT INTO accounts(email) VALUES (?1) ON CONFLICT(email) DO NOTHING",
        [email],
    )?;
    connection.query_row("SELECT id FROM accounts WHERE email = ?1", [email], |row| {
        row.get(0)
    })
}

fn save_snapshot(
    connection: &Connection,
    email: &str,
    snapshot: &EnergySnapshot,
) -> rusqlite::Result<()> {
    let transaction = connection.unchecked_transaction()?;
    let id = account_id(&transaction, email)?;
    transaction.execute(
        "INSERT INTO latest_snapshots (account_id,serial,observed_at,pv_watts,load_watts,grid_watts,battery_watts,battery_soc,solar_yield_kwh,updated_at,pv_to,to_load,to_grid,to_battery,battery_to,grid_to)
         VALUES (?1,?2,datetime('now'),?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
         ON CONFLICT(account_id,serial) DO UPDATE SET observed_at=excluded.observed_at,pv_watts=excluded.pv_watts,load_watts=excluded.load_watts,grid_watts=excluded.grid_watts,battery_watts=excluded.battery_watts,battery_soc=excluded.battery_soc,solar_yield_kwh=excluded.solar_yield_kwh,updated_at=excluded.updated_at,pv_to=excluded.pv_to,to_load=excluded.to_load,to_grid=excluded.to_grid,to_battery=excluded.to_battery,battery_to=excluded.battery_to,grid_to=excluded.grid_to",
        params![id, snapshot.inverter_sn, snapshot.pv_watts, snapshot.load_watts, snapshot.grid_watts, snapshot.battery_watts, snapshot.battery_soc, snapshot.solar_yield_kwh, snapshot.updated_at, snapshot.pv_to, snapshot.to_load, snapshot.to_grid, snapshot.to_battery, snapshot.battery_to, snapshot.grid_to],
    )?;
    transaction.commit()
}

fn load_snapshot(connection: &Connection, email: &str) -> rusqlite::Result<Option<EnergySnapshot>> {
    let id = connection
        .query_row("SELECT id FROM accounts WHERE email = ?1", [email], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?;
    let Some(id) = id else {
        return Ok(None);
    };
    connection.query_row("SELECT inverter_sn,pv_watts,load_watts,grid_watts,battery_watts,battery_soc,solar_yield_kwh,updated_at,pv_to,to_load,to_grid,to_battery,battery_to,grid_to FROM latest_snapshots WHERE account_id=?1 ORDER BY observed_at DESC LIMIT 1", [id], |row| Ok(EnergySnapshot { inverter_sn: row.get(0)?, pv_watts: row.get(1)?, load_watts: row.get(2)?, grid_watts: row.get(3)?, battery_watts: row.get(4)?, battery_soc: row.get(5)?, solar_yield_kwh: row.get(6)?, updated_at: row.get(7)?, pv_to: row.get(8)?, to_load: row.get(9)?, to_grid: row.get(10)?, to_battery: row.get(11)?, battery_to: row.get(12)?, grid_to: row.get(13)? })).optional()
}

fn save_history(
    connection: &Connection,
    email: &str,
    serial: &str,
    date: &str,
    history: &[HistorySeries],
) -> rusqlite::Result<()> {
    let transaction = connection.unchecked_transaction()?;
    let id = account_id(&transaction, email)?;
    insert_history_points(&transaction, id, serial, date, history)?;
    transaction.commit()
}

fn insert_history_points(
    transaction: &rusqlite::Transaction<'_>,
    account_id: i64,
    serial: &str,
    date: &str,
    history: &[HistorySeries],
) -> rusqlite::Result<()> {
    let mut statement = transaction.prepare(
        "INSERT INTO history_points(account_id,serial,point_time,label,watts,source_date,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,datetime('now')) ON CONFLICT(account_id,serial,source_date,point_time,label) DO UPDATE SET watts=excluded.watts,fetched_at=excluded.fetched_at",
    )?;
    for series in history {
        for point in &series.points {
            statement.execute(params![
                account_id,
                serial,
                point.time,
                series.label,
                point.watts,
                date
            ])?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn save_history_and_backfill(
    connection: &Connection,
    email: &str,
    serial: &str,
    date: &str,
    history: &[HistorySeries],
    next_date: &str,
    oldest_date: &str,
    status: &str,
    cancel_epoch: &AtomicU64,
    expected_epoch: u64,
) -> rusqlite::Result<()> {
    let transaction = connection.unchecked_transaction()?;
    let id = account_id(&transaction, email)?;
    insert_history_points(&transaction, id, serial, date, history)?;
    if cancel_epoch.load(Ordering::Acquire) != expected_epoch {
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }
    transaction.execute(
        "INSERT INTO backfill_state(account_id,serial,target_date,oldest_date,status,last_error,updated_at)
         VALUES (?1,?2,?3,?4,?5,NULL,datetime('now'))
         ON CONFLICT(account_id,serial) DO UPDATE SET target_date=excluded.target_date,oldest_date=excluded.oldest_date,status=excluded.status,last_error=NULL,updated_at=excluded.updated_at",
        params![id, serial, next_date, oldest_date, status],
    )?;
    transaction.commit()
}

fn load_backfill_state(
    connection: &Connection,
    email: &str,
    serial: &str,
) -> rusqlite::Result<Option<BackfillState>> {
    let Some(id) = connection
        .query_row("SELECT id FROM accounts WHERE email=?1", [email], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?
    else {
        return Ok(None);
    };
    connection
        .query_row(
            "SELECT target_date,status,last_error FROM backfill_state WHERE account_id=?1 AND serial=?2",
            params![id, serial],
            |row| {
                Ok(BackfillState {
                    target_date: row.get(0)?,
                    status: row.get(1)?,
                    last_error: row.get(2)?,
                })
            },
        )
        .optional()
}

fn save_backfill_state(
    connection: &Connection,
    email: &str,
    serial: &str,
    target_date: &str,
    oldest_date: &str,
    status: &str,
    last_error: Option<&str>,
) -> rusqlite::Result<()> {
    let id = account_id(connection, email)?;
    connection.execute(
        "INSERT INTO backfill_state(account_id,serial,target_date,oldest_date,status,last_error,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,datetime('now'))
         ON CONFLICT(account_id,serial) DO UPDATE SET target_date=excluded.target_date,oldest_date=excluded.oldest_date,status=excluded.status,last_error=excluded.last_error,updated_at=excluded.updated_at",
        params![id, serial, target_date, oldest_date, status, last_error],
    )?;
    Ok(())
}

fn load_history(
    connection: &Connection,
    email: &str,
    serial: Option<&str>,
    date: &str,
) -> rusqlite::Result<Vec<HistorySeries>> {
    let Some(serial) = serial else {
        return Ok(Vec::new());
    };
    let Some(id) = connection
        .query_row("SELECT id FROM accounts WHERE email = ?1", [email], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?
    else {
        return Ok(Vec::new());
    };
    let mut statement = connection.prepare(
        "SELECT label,point_time,watts FROM history_points WHERE account_id=?1 AND serial=?2 AND source_date=?3 ORDER BY point_time",
    )?;
    let mut rows = statement.query(params![id, serial, date])?;
    let mut history = Vec::<HistorySeries>::new();
    while let Some(row) = rows.next()? {
        let label: String = row.get(0)?;
        let point = HistoryPoint {
            time: row.get(1)?,
            watts: row.get(2)?,
        };
        if let Some(series) = history.iter_mut().find(|series| series.label == label) {
            series.points.push(point);
        } else {
            history.push(HistorySeries {
                label,
                points: vec![point],
            });
        }
    }
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::{
        configure, load_backfill_state, load_history, save_history, save_history_and_backfill,
    };
    use crate::domain::{HistoryPoint, HistorySeries};
    use rusqlite::Connection;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    #[test]
    fn history_keeps_same_time_points_from_different_dates() {
        let connection = Connection::open_in_memory().unwrap();
        configure(&connection).unwrap();
        let first = vec![HistorySeries {
            label: "pv".into(),
            points: vec![HistoryPoint {
                time: "12:00".into(),
                watts: 100.0,
            }],
        }];
        let second = vec![HistorySeries {
            label: "pv".into(),
            points: vec![HistoryPoint {
                time: "12:00".into(),
                watts: 200.0,
            }],
        }];
        save_history(
            &connection,
            "demo@example.com",
            "serial",
            "2026-09-07",
            &first,
        )
        .unwrap();
        save_history(
            &connection,
            "demo@example.com",
            "serial",
            "2026-09-08",
            &second,
        )
        .unwrap();

        assert_eq!(
            load_history(
                &connection,
                "demo@example.com",
                Some("serial"),
                "2026-09-07"
            )
            .unwrap()[0]
                .points[0]
                .watts,
            100.0
        );
        assert_eq!(
            load_history(
                &connection,
                "demo@example.com",
                Some("serial"),
                "2026-09-08"
            )
            .unwrap()[0]
                .points[0]
                .watts,
            200.0
        );
    }

    #[test]
    fn canceled_backfill_rolls_back_history_and_state() {
        let connection = Connection::open_in_memory().unwrap();
        configure(&connection).unwrap();
        let epoch = Arc::new(AtomicU64::new(1));
        let result = save_history_and_backfill(
            &connection,
            "test@example.com",
            "serial",
            "2026-09-07",
            &[HistorySeries {
                label: "PV".into(),
                points: vec![HistoryPoint {
                    time: "12:00".into(),
                    watts: 1.0,
                }],
            }],
            "2026-09-06",
            "2025-09-08",
            "running",
            &epoch,
            0,
        );
        assert!(result.is_err());
        assert!(load_history(
            &connection,
            "test@example.com",
            Some("serial"),
            "2026-09-07"
        )
        .unwrap()
        .is_empty());
        let state_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM backfill_state", [], |row| row.get(0))
            .unwrap();
        assert_eq!(state_count, 0);
        assert_eq!(epoch.load(Ordering::Acquire), 1);
    }

    #[test]
    fn empty_history_still_advances_backfill_state() {
        let connection = Connection::open_in_memory().unwrap();
        configure(&connection).unwrap();
        let epoch = Arc::new(AtomicU64::new(0));

        save_history_and_backfill(
            &connection,
            "demo@example.com",
            "serial",
            "2026-09-07",
            &[],
            "2026-09-06",
            "2025-09-08",
            "running",
            &epoch,
            0,
        )
        .unwrap();

        assert!(load_history(
            &connection,
            "demo@example.com",
            Some("serial"),
            "2026-09-07"
        )
        .unwrap()
        .is_empty());
        let state = load_backfill_state(&connection, "demo@example.com", "serial")
            .unwrap()
            .unwrap();
        assert_eq!(state.target_date, "2026-09-06");
        assert_eq!(state.status, "running");
    }
}
