-- Charge codes
CREATE TABLE charge_codes (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL,
    description TEXT NOT NULL,
    open TEXT NOT NULL,
    close TEXT
);

-- Active charge codes must be unique
CREATE UNIQUE INDEX active_charge_codes
ON charge_codes(code)
WHERE close IS NULL;

-- Projects
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    default_charge_code INTEGER REFERENCES charge_codes(id),
    open TEXT NOT NULL,
    close TEXT
);

-- Active projects must be unique
CREATE UNIQUE INDEX active_projects
ON projects(title)
WHERE close IS NULL;

-- Tasks
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    project INTEGER REFERENCES projects(id),
    description TEXT NOT NULL,
    charge_code INTEGER REFERENCES charge_codes(id),
    open TEXT NOT NULL,
    close TEXT        
);

-- Work logs
CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    task INTEGER REFERENCES tasks(id),
    start TEXT NOT NULL,
    stop TEXT
);
