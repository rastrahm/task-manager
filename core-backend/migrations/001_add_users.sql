-- Migración para bases de datos creadas antes de la tabla users.
-- Ejecutar manualmente si ya existía la tabla tasks sin user_id.

BEGIN;

CREATE TABLE IF NOT EXISTS users (
    id            SERIAL PRIMARY KEY,
    username      VARCHAR(64) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin      BOOLEAN NOT NULL DEFAULT FALSE,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Usuario por defecto para reasignar tareas huérfanas (cambiar password_hash después).
INSERT INTO users (username, password_hash, is_admin)
VALUES ('admin', 'PENDING_HASH', TRUE)
ON CONFLICT (username) DO NOTHING;

ALTER TABLE tasks
    ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;

UPDATE tasks
SET user_id = (SELECT id FROM users WHERE username = 'admin' LIMIT 1)
WHERE user_id IS NULL;

ALTER TABLE tasks
    ALTER COLUMN user_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks (user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks (parent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_user_parent ON tasks (user_id, parent_id);

COMMIT;
