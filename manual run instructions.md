# InsightBoard Manual Run Instructions

## 1) Start From Repo Root

```bash
cd /home/gui-13/PersProj/InsightBoard
```

## 2) Prepare Environment File

```bash
cp /home/gui-13/PersProj/InsightBoard/.env.example /home/gui-13/PersProj/InsightBoard/.env
```

Edit `.env` and ensure these are set:
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET`

Optional widget keys:
- `OPENWEATHER_API_KEY`
- `NEWSAPI_API_KEY`
- `GITHUB_API_TOKEN`

## 3) Start Infrastructure (Postgres + Redis)

```bash
sudo docker compose up -d
sudo docker compose ps
```

Expected: `insightboard-postgres` and `insightboard-redis` are `healthy`.

## 4) Install Frontend Dependencies

```bash
cd /home/gui-13/PersProj/InsightBoard/frontend
npm ci
```

## 5) Run Backend (Terminal A)

```bash
cd /home/gui-13/PersProj/InsightBoard/backend
cargo run
```

Keep this terminal running.

## 6) Run Frontend (Terminal B)

```bash
cd /home/gui-13/PersProj/InsightBoard/frontend
npm run dev -- --host 127.0.0.1 --port 5173
```

Open:
- `http://127.0.0.1:5173`

## 7) Manual UI Test Checklist

1. Register at `/register`.
2. Create dashboard.
3. Open dashboard editor.
4. Add `Status` widget and configure `https://example.com`.
5. Drag/resize widget, wait ~2s, refresh page, verify layout persists.
6. Add Weather + News widgets:
   - No keys: expect clear config errors.
   - Keys set + backend restarted: expect live data.
7. Add Crypto widget with `bitcoin,ethereum`.
8. Return to dashboard list and verify dashboard exists.
9. Delete dashboard and verify removal.

## 8) Manual API Test (Curl)

```bash
BASE=http://127.0.0.1:8080
curl -i "$BASE/healthz"
```

Register and capture token:

```bash
EMAIL="manual_$(date +%s)@example.com"
PASS='password123'
REG=$(curl -sS -H 'Content-Type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASS\"}" \
  "$BASE/api/auth/register")
echo "$REG"
TOKEN=$(printf '%s' "$REG" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
```

Current user:

```bash
curl -i -H "Authorization: Bearer $TOKEN" "$BASE/api/me"
```

Dashboard CRUD:

```bash
CREATE=$(curl -sS -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Manual API Dashboard","layout_json":{"widgets":[]},"settings_json":{"title":"Manual API Dashboard","theme":"light"}}' \
  "$BASE/api/dashboards")
echo "$CREATE"
DID=$(printf '%s' "$CREATE" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

curl -i -H "Authorization: Bearer $TOKEN" "$BASE/api/dashboards/$DID"
curl -i -X PUT -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Manual API Dashboard"}' \
  "$BASE/api/dashboards/$DID"
curl -i -X DELETE -H "Authorization: Bearer $TOKEN" "$BASE/api/dashboards/$DID"
```

Widget endpoints:

```bash
curl -i -H "Authorization: Bearer $TOKEN" \
  "$BASE/api/data/status?urls=https://example.com,https://httpstat.us/503"

curl -i -H "Authorization: Bearer $TOKEN" \
  "$BASE/api/data/weather?city=Chicago"

curl -i -H "Authorization: Bearer $TOKEN" \
  "$BASE/api/data/news?topic=technology"

curl -i -H "Authorization: Bearer $TOKEN" \
  "$BASE/api/data/crypto?ids=bitcoin,ethereum"

curl -i -H "Authorization: Bearer $TOKEN" \
  "$BASE/api/data/github?username=octocat"
```

Notes:
- Weather/News return `400` if API keys are not configured.
- Crypto/GitHub may fail due to external provider/network limits.

## 9) Optional Verification Commands

Backend tests:

```bash
cd /home/gui-13/PersProj/InsightBoard/backend
cargo test
```

Frontend build:

```bash
cd /home/gui-13/PersProj/InsightBoard/frontend
npm run build
```

Audit dependencies:

```bash
cd /home/gui-13/PersProj/InsightBoard/frontend
npm audit
```

## 10) Inspect Database Entries (Auth, Dashboards)

Open Postgres shell inside the container:

```bash
cd /home/gui-13/PersProj/InsightBoard
sudo docker compose exec -T postgres psql -U postgres -d insightboard
```

Inside `psql`:

```sql
\dt
SELECT id, email, password_hash, created_at
FROM users
ORDER BY created_at DESC
LIMIT 20;

SELECT id, user_id, name, created_at, updated_at
FROM dashboards
ORDER BY updated_at DESC
LIMIT 20;
\q
```

One-liner (non-interactive) users query:

```bash
sudo docker compose exec -T postgres psql -U postgres -d insightboard -c "SELECT id, email, created_at FROM users ORDER BY created_at DESC LIMIT 20;"
```

## 11) Stop Everything

Stop backend/frontend with `Ctrl+C` in each terminal.

Stop infra:

```bash
cd /home/gui-13/PersProj/InsightBoard
sudo docker compose down
```

Full reset (remove volumes/data):

```bash
sudo docker compose down -v
```
