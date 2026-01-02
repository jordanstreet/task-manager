-- Charge codes
CREATE TABLE IF NOT EXISTS charge_codes (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL,
    details TEXT NOT NULL,
    open TEXT NOT NULL,
    close TEXT
);

-- Active charge codes must be unique
CREATE UNIQUE INDEX IF NOT EXISTS active_charge_codes
ON charge_codes(code)
WHERE close IS NULL;

-- Work logs
CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY,
    description TEXT,
    start TEXT NOT NULL,
    stop TEXT
);

-- Work log charge code links
CREATE TABLE IF NOT EXISTS log_charge_codes (
    log_id INTEGER NOT NULL,
    charge_code_id INTEGER NOT NULL,
    PRIMARY KEY (log_id, charge_code_id),
    FOREIGN KEY (log_id) REFERENCES logs(id) ON DELETE CASCADE,
    FOREIGN KEY (charge_code_id) REFERENCES charge_codes(id) ON DELETE RESTRICT
);
