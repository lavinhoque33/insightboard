-- M4b (D8, D17, D29): dashboard JSON defaults match the contract, stored values are backfilled,
-- emails are normalised, and the redundant email index is dropped.

-- The contract's empty layout is {"widgets":[]}, not [].
ALTER TABLE dashboards ALTER COLUMN layout_json SET DEFAULT '{"widgets":[]}'::jsonb;

UPDATE dashboards
SET layout_json = '{"widgets":[]}'::jsonb
WHERE layout_json = 'null'::jsonb OR layout_json = '[]'::jsonb;

UPDATE dashboards
SET settings_json = '{}'::jsonb
WHERE settings_json = 'null'::jsonb;

-- Register and login now trim and lower-case emails. Normalise stored ones unless that would
-- collide with another account (such rows are left unchanged rather than merged).
UPDATE users u
SET email = lower(btrim(u.email))
WHERE u.email <> lower(btrim(u.email))
  AND NOT EXISTS (
      SELECT 1 FROM users o
      WHERE o.id <> u.id AND lower(btrim(o.email)) = lower(btrim(u.email))
  );

-- users.email is UNIQUE, which already creates an index; this one duplicated it.
DROP INDEX IF EXISTS idx_users_email;
