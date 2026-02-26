PRAGMA foreign_keys = ON;

-- ========================
-- SERVER
-- ========================
CREATE TABLE server (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT NOT NULL UNIQUE,
    address             TEXT NOT NULL,
    username            TEXT NOT NULL,
    remote_directory    TEXT NOT NULL
);

-- ========================
-- PROJECT
-- ========================
CREATE TABLE project (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    created_at              TEXT NOT NULL,
    local_directory         TEXT NOT NULL,
    src_directory           TEXT NOT NULL,
    post_files_json         TEXT NOT NULL,
    get_files_json          TEXT NOT NULL
);

-- ========================
-- RUN
-- ========================
CREATE TABLE run (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id              INTEGER NOT NULL,

    name                    TEXT    NOT NULL UNIQUE,
    notes                   TEXT    NOT NULL,
    config_json             TEXT    NOT NULL,
    created_at              TEXT    NOT NULL,

    server_id               INTEGER NOT NULL,
    remote_directory        TEXT    NOT NULL,
    local_directory         TEXT    NOT NULL,

    post_files_json         TEXT NOT NULL,
    get_files_json          TEXT NOT NULL,

    FOREIGN KEY(project_id) REFERENCES project(id) ON DELETE CASCADE,
    FOREIGN KEY(server_id) REFERENCES server(id) ON DELETE RESTRICT
);

-- ========================
-- INDEXES
-- ========================
CREATE INDEX idx_run_project ON run(project_id);
CREATE INDEX idx_run_server  ON run(server_id);
