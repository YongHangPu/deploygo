use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::config::{derive_releases_root_path, DeployRecord, Project, ServerConfig};
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginatedResult<T: serde::Serialize> {
    pub data: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportStats {
    pub imported_servers: u32,
    pub skipped_servers: u32,
    pub imported_projects: u32,
    pub skipped_projects: u32,
    pub imported_passwords: u32,
}

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HistoryFilter {
    pub project_id: Option<String>,
    pub server_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)?;

        if has_legacy_columns(&conn)? {
            conn.execute_batch(
                "DROP TABLE IF EXISTS deploy_history;
                 DROP TABLE IF EXISTS passwords;
                 DROP TABLE IF EXISTS projects;
                 DROP TABLE IF EXISTS servers;",
            )?;
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                local_dist_path TEXT NOT NULL,
                build_command   TEXT,
                server_id       TEXT NOT NULL,
                keep_versions   INTEGER NOT NULL DEFAULT 5,
                live_root_path  TEXT NOT NULL,
                releases_root_path TEXT NOT NULL,
                sort_order      INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS servers (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                host            TEXT NOT NULL,
                port            INTEGER NOT NULL DEFAULT 22,
                username        TEXT NOT NULL,
                auth_method     TEXT NOT NULL DEFAULT 'password',
                key_path        TEXT,
                sort_order      INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS passwords (
                server_id       TEXT PRIMARY KEY,
                password_base64 TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS deploy_history (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id      TEXT NOT NULL,
                project_name    TEXT NOT NULL,
                server_id       TEXT NOT NULL,
                server_name     TEXT NOT NULL,
                release_dir_path TEXT NOT NULL,
                live_root_path  TEXT NOT NULL,
                version         TEXT,
                artifact_fingerprint TEXT,
                status          TEXT NOT NULL DEFAULT 'running',
                file_count      INTEGER NOT NULL DEFAULT 0,
                total_size      INTEGER NOT NULL DEFAULT 0,
                exit_code       INTEGER,
                duration_ms     INTEGER NOT NULL DEFAULT 0,
                log             TEXT NOT NULL DEFAULT '',
                created_at      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_history_project_id ON deploy_history(project_id);
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON deploy_history(created_at);
            CREATE INDEX IF NOT EXISTS idx_history_status ON deploy_history(status);
            ",
        )?;
        ensure_deploy_history_columns(&conn)?;
        ensure_servers_columns(&conn)?;
        ensure_projects_columns(&conn)?;

        let db = Database {
            conn: Mutex::new(conn),
        };

        Ok(db)
    }

    // ====== 项目 ======

    pub fn get_projects(&self) -> Result<Vec<Project>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, local_dist_path, build_command, server_id, keep_versions,
                    live_root_path, releases_root_path, sort_order, created_at, updated_at
             FROM projects ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                local_dist_path: row.get(2)?,
                build_command: row.get(3)?,
                server_id: row.get(4)?,
                keep_versions: row.get(5)?,
                live_root_path: row.get(6)?,
                releases_root_path: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(hydrate_project(row?));
        }
        Ok(projects)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, local_dist_path, build_command, server_id, keep_versions,
                    live_root_path, releases_root_path, sort_order, created_at, updated_at
             FROM projects WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                local_dist_path: row.get(2)?,
                build_command: row.get(3)?,
                server_id: row.get(4)?,
                keep_versions: row.get(5)?,
                live_root_path: row.get(6)?,
                releases_root_path: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(Ok(project)) => Ok(Some(hydrate_project(project))),
            _ => Ok(None),
        }
    }

    pub fn save_project(&self, project: &Project) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (id, name, local_dist_path, build_command, server_id, keep_versions, live_root_path, releases_root_path, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, local_dist_path=excluded.local_dist_path, build_command=excluded.build_command,
                server_id=excluded.server_id, keep_versions=excluded.keep_versions, live_root_path=excluded.live_root_path,
                releases_root_path=excluded.releases_root_path, sort_order=excluded.sort_order, updated_at=excluded.updated_at",
            rusqlite::params![project.id, project.name, project.local_dist_path, project.build_command, project.server_id,
                project.keep_versions, project.live_root_path, project.releases_root_path, project.sort_order,
                project.created_at, project.updated_at],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn reorder_projects(&self, ids: &[String]) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE projects SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i32, id],
            )?;
        }
        Ok(())
    }

    // ====== 服务器 ======

    pub fn get_servers(&self) -> Result<Vec<ServerConfig>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, auth_method, key_path,
                    jump_host, jump_port, jump_username, jump_auth_method, jump_key_path,
                    sort_order, created_at, updated_at
             FROM servers ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ServerConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                auth_method: row.get(5)?,
                key_path: row.get(6)?,
                jump_host: row.get(7)?,
                jump_port: row.get(8)?,
                jump_username: row.get(9)?,
                jump_auth_method: row.get(10)?,
                jump_key_path: row.get(11)?,
                sort_order: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;
        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }
        Ok(servers)
    }

    pub fn get_server(&self, id: &str) -> Result<Option<ServerConfig>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, auth_method, key_path,
                    jump_host, jump_port, jump_username, jump_auth_method, jump_key_path,
                    sort_order, created_at, updated_at
             FROM servers WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(ServerConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                auth_method: row.get(5)?,
                key_path: row.get(6)?,
                jump_host: row.get(7)?,
                jump_port: row.get(8)?,
                jump_username: row.get(9)?,
                jump_auth_method: row.get(10)?,
                jump_key_path: row.get(11)?,
                sort_order: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;
        match rows.next() {
            Some(Ok(server)) => Ok(Some(server)),
            _ => Ok(None),
        }
    }

    pub fn save_server(&self, server: &ServerConfig) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO servers (id, name, host, port, username, auth_method, key_path,
                    jump_host, jump_port, jump_username, jump_auth_method, jump_key_path,
                    sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, host=excluded.host, port=excluded.port, username=excluded.username,
                auth_method=excluded.auth_method, key_path=excluded.key_path,
                jump_host=excluded.jump_host, jump_port=excluded.jump_port,
                jump_username=excluded.jump_username, jump_auth_method=excluded.jump_auth_method,
                jump_key_path=excluded.jump_key_path, sort_order=excluded.sort_order,
                updated_at=excluded.updated_at",
            rusqlite::params![server.id, server.name, server.host, server.port, server.username,
                server.auth_method, server.key_path,
                server.jump_host, server.jump_port, server.jump_username,
                server.jump_auth_method, server.jump_key_path, server.sort_order,
                server.created_at, server.updated_at],
        )?;
        Ok(())
    }

    pub fn delete_server(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM servers WHERE id = ?1", rusqlite::params![id])?;
        conn.execute(
            "DELETE FROM passwords WHERE server_id = ?1",
            rusqlite::params![id],
        )?;
        // 同时删除跳板机密码（使用 :jump 后缀存储）。
        conn.execute(
            "DELETE FROM passwords WHERE server_id = ?1",
            rusqlite::params![format!("{}:jump", id)],
        )?;
        Ok(())
    }

    pub fn reorder_servers(&self, ids: &[String]) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE servers SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i32, id],
            )?;
        }
        Ok(())
    }

    // ====== 密码 ======

    /// 构造 passwords 表使用的键。
    /// 主服务器密码直接使用 server_id；跳板机密码追加 ":jump"。
    pub(crate) fn password_key(server_id: &str, tag: &str) -> String {
        if tag.is_empty() || tag == "main" {
            server_id.to_string()
        } else {
            format!("{}:{}", server_id, tag)
        }
    }

    pub fn store_password_tagged(
        &self,
        server_id: &str,
        password_base64: &str,
        tag: &str,
    ) -> Result<(), AppError> {
        let key = Database::password_key(server_id, tag);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO passwords (server_id, password_base64) VALUES (?1, ?2)
             ON CONFLICT(server_id) DO UPDATE SET password_base64=excluded.password_base64",
            rusqlite::params![key, password_base64],
        )?;
        Ok(())
    }

    pub fn get_password_tagged(
        &self,
        server_id: &str,
        tag: &str,
    ) -> Result<Option<String>, AppError> {
        let key = Database::password_key(server_id, tag);
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT password_base64 FROM passwords WHERE server_id = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(pwd)) => Ok(Some(pwd)),
            _ => Ok(None),
        }
    }

    pub fn store_password(&self, server_id: &str, password_base64: &str) -> Result<(), AppError> {
        self.store_password_tagged(server_id, password_base64, "main")
    }

    pub fn get_password(&self, server_id: &str) -> Result<Option<String>, AppError> {
        self.get_password_tagged(server_id, "main")
    }

    pub fn delete_password_tagged(&self, server_id: &str, tag: &str) -> Result<(), AppError> {
        let key = Self::password_key(server_id, tag);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM passwords WHERE server_id = ?1",
            rusqlite::params![key],
        )?;
        Ok(())
    }

    pub fn import_configuration(
        &self,
        servers: &[ServerConfig],
        projects: &[Project],
        passwords: &HashMap<String, String>,
        jump_passwords: &HashMap<String, String>,
        overwrite: bool,
    ) -> Result<ImportStats, AppError> {
        let mut conn = self.conn.lock().unwrap();
        let transaction = conn.transaction()?;
        let mut stats = ImportStats::default();

        for server in servers {
            let sql = if overwrite {
                "INSERT INTO servers (id, name, host, port, username, auth_method, key_path,
                    jump_host, jump_port, jump_username, jump_auth_method, jump_key_path,
                    sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(id) DO UPDATE SET
                    name=excluded.name, host=excluded.host, port=excluded.port,
                    username=excluded.username, auth_method=excluded.auth_method,
                    key_path=excluded.key_path, jump_host=excluded.jump_host,
                    jump_port=excluded.jump_port, jump_username=excluded.jump_username,
                    jump_auth_method=excluded.jump_auth_method,
                    jump_key_path=excluded.jump_key_path, sort_order=excluded.sort_order,
                    updated_at=excluded.updated_at"
            } else {
                "INSERT OR IGNORE INTO servers (id, name, host, port, username, auth_method,
                    key_path, jump_host, jump_port, jump_username, jump_auth_method,
                    jump_key_path, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"
            };
            let changed = transaction.execute(
                sql,
                rusqlite::params![
                    server.id,
                    server.name,
                    server.host,
                    server.port,
                    server.username,
                    server.auth_method,
                    server.key_path,
                    server.jump_host,
                    server.jump_port,
                    server.jump_username,
                    server.jump_auth_method,
                    server.jump_key_path,
                    server.sort_order,
                    server.created_at,
                    server.updated_at,
                ],
            )?;
            if changed == 0 {
                stats.skipped_servers += 1;
            } else {
                stats.imported_servers += 1;
            }
        }

        for project in projects {
            let sql = if overwrite {
                "INSERT INTO projects (id, name, local_dist_path, build_command, server_id,
                    keep_versions, live_root_path, releases_root_path, sort_order,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(id) DO UPDATE SET
                    name=excluded.name, local_dist_path=excluded.local_dist_path,
                    build_command=excluded.build_command, server_id=excluded.server_id,
                    keep_versions=excluded.keep_versions, live_root_path=excluded.live_root_path,
                    releases_root_path=excluded.releases_root_path,
                    sort_order=excluded.sort_order, updated_at=excluded.updated_at"
            } else {
                "INSERT OR IGNORE INTO projects (id, name, local_dist_path, build_command,
                    server_id, keep_versions, live_root_path, releases_root_path, sort_order,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            };
            let changed = transaction.execute(
                sql,
                rusqlite::params![
                    project.id,
                    project.name,
                    project.local_dist_path,
                    project.build_command,
                    project.server_id,
                    project.keep_versions,
                    project.live_root_path,
                    project.releases_root_path,
                    project.sort_order,
                    project.created_at,
                    project.updated_at,
                ],
            )?;
            if changed == 0 {
                stats.skipped_projects += 1;
            } else {
                stats.imported_projects += 1;
            }
        }

        for (server_id, encrypted) in passwords {
            stats.imported_passwords +=
                import_password_row(&transaction, server_id, encrypted, overwrite)? as u32;
        }
        for (server_id, encrypted) in jump_passwords {
            stats.imported_passwords += import_password_row(
                &transaction,
                &Self::password_key(server_id, "jump"),
                encrypted,
                overwrite,
            )? as u32;
        }

        transaction.commit()?;
        Ok(stats)
    }

    // ====== 部署历史 ======

    /// 写入部署历史记录。仅由完整版部署引擎（overlay 注入）调用。
    #[allow(dead_code)]
    pub fn insert_record(&self, record: &DeployRecord) -> Result<i64, AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO deploy_history (project_id, project_name, server_id, server_name,
                release_dir_path, live_root_path, version, artifact_fingerprint, status, file_count, total_size,
                exit_code, duration_ms, log, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                record.project_id,
                record.project_name,
                record.server_id,
                record.server_name,
                record.release_dir_path,
                record.live_root_path,
                record.version,
                record.artifact_fingerprint,
                record.status,
                record.file_count,
                record.total_size,
                record.exit_code,
                record.duration_ms,
                record.log,
                record.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_record(&self, id: i64) -> Result<Option<DeployRecord>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, project_name, server_id, server_name,
                release_dir_path, live_root_path, version, artifact_fingerprint, status,
                file_count, total_size, exit_code, duration_ms, log, created_at
            FROM deploy_history WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(DeployRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                project_name: row.get(2)?,
                server_id: row.get(3)?,
                server_name: row.get(4)?,
                release_dir_path: row.get(5)?,
                live_root_path: row.get(6)?,
                version: row.get(7)?,
                artifact_fingerprint: row.get(8)?,
                status: row.get(9)?,
                file_count: row.get(10)?,
                total_size: row.get(11)?,
                exit_code: row.get(12)?,
                duration_ms: row.get(13)?,
                log: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            _ => Ok(None),
        }
    }

    pub fn get_history(
        &self,
        filter: &HistoryFilter,
    ) -> Result<PaginatedResult<DeployRecord>, AppError> {
        let conn = self.conn.lock().unwrap();

        let mut where_sql = String::from("WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref pid) = filter.project_id {
            where_sql.push_str(" AND project_id = ?");
            params.push(Box::new(pid.clone()));
        }
        if let Some(ref sid) = filter.server_id {
            where_sql.push_str(" AND server_id = ?");
            params.push(Box::new(sid.clone()));
        }
        if let Some(ref s) = filter.status {
            where_sql.push_str(" AND status = ?");
            params.push(Box::new(s.clone()));
        }
        if let Some(ref d) = filter.date_from {
            where_sql.push_str(" AND created_at >= ?");
            params.push(Box::new(d.clone()));
        }
        if let Some(ref d) = filter.date_to {
            where_sql.push_str(" AND created_at <= ?");
            params.push(Box::new(format!("{}T23:59:59.999Z", d)));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        let count_sql = format!("SELECT COUNT(*) FROM deploy_history {}", where_sql);
        let mut count_stmt = conn.prepare(&count_sql)?;
        let total: i64 = count_stmt.query_row(param_refs.as_slice(), |row| row.get(0))?;

        let limit = filter.limit.unwrap_or(20);
        let offset = filter.offset.unwrap_or(0);
        let data_sql = format!(
            "SELECT id, project_id, project_name, server_id, server_name,
                release_dir_path, live_root_path, version, artifact_fingerprint, status,
                file_count, total_size, exit_code, duration_ms, log, created_at
            FROM deploy_history {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_sql
        );

        let mut data_params = params;
        data_params.push(Box::new(limit));
        data_params.push(Box::new(offset));
        let data_param_refs: Vec<&dyn rusqlite::types::ToSql> =
            data_params.iter().map(|p| p.as_ref()).collect();

        let mut data_stmt = conn.prepare(&data_sql)?;
        let rows = data_stmt.query_map(data_param_refs.as_slice(), |row| {
            Ok(DeployRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                project_name: row.get(2)?,
                server_id: row.get(3)?,
                server_name: row.get(4)?,
                release_dir_path: row.get(5)?,
                live_root_path: row.get(6)?,
                version: row.get(7)?,
                artifact_fingerprint: row.get(8)?,
                status: row.get(9)?,
                file_count: row.get(10)?,
                total_size: row.get(11)?,
                exit_code: row.get(12)?,
                duration_ms: row.get(13)?,
                log: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(PaginatedResult {
            data: records,
            total,
        })
    }

    /// 查询项目最近一次成功部署记录。仅由完整版部署引擎（overlay 注入）调用。
    #[allow(dead_code)]
    pub fn get_latest_effective_record(
        &self,
        project_id: &str,
    ) -> Result<Option<DeployRecord>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, project_name, server_id, server_name,
                release_dir_path, live_root_path, version, artifact_fingerprint, status,
                file_count, total_size, exit_code, duration_ms, log, created_at
            FROM deploy_history
            WHERE project_id = ?1 AND status IN ('success', 'rollback')
            ORDER BY created_at DESC
            LIMIT 1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok(DeployRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                project_name: row.get(2)?,
                server_id: row.get(3)?,
                server_name: row.get(4)?,
                release_dir_path: row.get(5)?,
                live_root_path: row.get(6)?,
                version: row.get(7)?,
                artifact_fingerprint: row.get(8)?,
                status: row.get(9)?,
                file_count: row.get(10)?,
                total_size: row.get(11)?,
                exit_code: row.get(12)?,
                duration_ms: row.get(13)?,
                log: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            _ => Ok(None),
        }
    }

    pub fn delete_record(&self, id: i64) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM deploy_history WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}

fn hydrate_project(mut project: Project) -> Project {
    if project.releases_root_path.trim().is_empty() {
        if let Ok(derived) = derive_releases_root_path(&project.live_root_path) {
            project.releases_root_path = derived;
        }
    }

    project
}

fn import_password_row(
    transaction: &rusqlite::Transaction<'_>,
    key: &str,
    encrypted: &str,
    overwrite: bool,
) -> Result<usize, AppError> {
    let sql = if overwrite {
        "INSERT INTO passwords (server_id, password_base64) VALUES (?1, ?2)
         ON CONFLICT(server_id) DO UPDATE SET password_base64=excluded.password_base64"
    } else {
        "INSERT OR IGNORE INTO passwords (server_id, password_base64) VALUES (?1, ?2)"
    };
    Ok(transaction.execute(sql, rusqlite::params![key, encrypted])?)
}

fn has_legacy_columns(conn: &Connection) -> Result<bool, AppError> {
    Ok(table_has_column(conn, "projects", "source_subdir")?
        || table_has_column(conn, "servers", "deploy_script_path")?)
}

fn ensure_deploy_history_columns(conn: &Connection) -> Result<(), AppError> {
    if !table_has_column(conn, "deploy_history", "artifact_fingerprint")? {
        conn.execute(
            "ALTER TABLE deploy_history ADD COLUMN artifact_fingerprint TEXT",
            [],
        )?;
    }

    Ok(())
}

fn ensure_projects_columns(conn: &Connection) -> Result<(), AppError> {
    if !table_has_column(conn, "projects", "sort_order")? {
        conn.execute(
            "ALTER TABLE projects ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

fn ensure_servers_columns(conn: &Connection) -> Result<(), AppError> {
    if !table_has_column(conn, "servers", "key_path")? {
        conn.execute("ALTER TABLE servers ADD COLUMN key_path TEXT", [])?;
    }

    if !table_has_column(conn, "servers", "jump_host")? {
        conn.execute("ALTER TABLE servers ADD COLUMN jump_host TEXT", [])?;
    }
    if !table_has_column(conn, "servers", "jump_port")? {
        conn.execute("ALTER TABLE servers ADD COLUMN jump_port INTEGER", [])?;
    }
    if !table_has_column(conn, "servers", "jump_username")? {
        conn.execute("ALTER TABLE servers ADD COLUMN jump_username TEXT", [])?;
    }
    if !table_has_column(conn, "servers", "jump_auth_method")? {
        conn.execute("ALTER TABLE servers ADD COLUMN jump_auth_method TEXT", [])?;
    }
    if !table_has_column(conn, "servers", "jump_key_path")? {
        conn.execute("ALTER TABLE servers ADD COLUMN jump_key_path TEXT", [])?;
    }
    if !table_has_column(conn, "servers", "sort_order")? {
        conn.execute(
            "ALTER TABLE servers ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, AppError> {
    let sql = format!(
        "SELECT 1 FROM pragma_table_info('{}') WHERE name = ?1 LIMIT 1",
        table
    );
    let mut stmt = conn.prepare(&sql)?;
    let exists = stmt.exists(rusqlite::params![column])?;
    Ok(exists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_db(dir: &Path) -> Database {
        let db_path = dir.join("deploygo.db");
        Database::new(&db_path).expect("failed to create test db")
    }

    fn sample_server(id: &str) -> ServerConfig {
        ServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            host: "example.com".to_string(),
            port: 22,
            username: "deploy".to_string(),
            auth_method: "password".to_string(),
            key_path: None,
            jump_host: None,
            jump_port: None,
            jump_username: None,
            jump_auth_method: None,
            jump_key_path: None,
            sort_order: 0,
            created_at: "2026-07-27T00:00:00Z".to_string(),
            updated_at: "2026-07-27T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn configuration_import_rolls_back_all_rows_on_failure() {
        let dir = TempDir::new().unwrap();
        let db = setup_db(dir.path());
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "CREATE TRIGGER reject_bad_import BEFORE INSERT ON servers
                 WHEN NEW.id = 'bad' BEGIN SELECT RAISE(ABORT, 'rejected'); END;",
            )
            .unwrap();
        }

        let error = db
            .import_configuration(
                &[sample_server("good"), sample_server("bad")],
                &[],
                &HashMap::new(),
                &HashMap::new(),
                false,
            )
            .unwrap_err();

        assert!(error.message.contains("rejected"));
        assert!(db.get_server("good").unwrap().is_none());
    }
}
