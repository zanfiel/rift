<div align="center">

# Rift

### Your chat. Your rules. Your data.

A Discord-compatible chat platform with a built-in bridge that tears open a portal
between Discord and your own self-hosted server. Full message history, zero telemetry,
no Nitro restrictions.

[Why Rift?](#why-rift) · [Architecture](#architecture) · [Quick Start](#quick-start) · [Discord Bridge](#discord-bridge) · [API](#api-reference) · [Self-Host](#self-hosting)

</div>

---

## Why Rift?

Discord owns your messages, your data, your social graph. They can ban you, delete your history, and you have zero recourse. Nitro paywalls basic features. Telemetry tracks everything.

Rift is the exit.

- **🌀 Discord Bridge** — connect to Discord through Rift. Your friends don't need to switch. Messages flow both ways, but *your* copy lives on *your* server.
- **📜 Full message history** — every message stored in PostgreSQL with full-text search. Edits and deletes are logged in an audit trail — what Discord hides, Rift preserves.
- **🔓 No Nitro restrictions** — upload any size file, use any emoji, send messages of any length (auto-split for Discord delivery).
- **🛡️ Zero telemetry** — no tracking, no analytics, no data harvesting. Your conversations stay on your hardware.
- **🤖 AI-ready** — local message history means you can plug in summarization, translation, smart notifications, or any LLM pipeline.
- **⚡ Blazingly fast** — Rust server (Axum + Tokio), Svelte 5 frontend. Sub-millisecond message routing.

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│  Rift Client │ ◄─ws──► │  Rift Server  │ ◄─ws──► │   Discord    │
│  (Svelte 5)  │         │  (Axum/Rust)  │         │   Gateway    │
└─────────────┘         │              │         └─────────────┘
                         │  PostgreSQL   │
                         │  Full History  │
                         └──────────────┘
```

---

## What's Built

### Server — `rift-server` (Rust)

| Feature | Status |
|---------|--------|
| Auth (JWT + Argon2 + refresh token rotation) | ✅ |
| Users (CRUD, profile, avatar, presence) | ✅ |
| Servers/guilds (CRUD, member management) | ✅ |
| Channels (text/voice, ordering, topics) | ✅ |
| Messages (CRUD, pagination, attachments) | ✅ |
| Direct messages (create DM, send/list) | ✅ |
| Roles (Discord-style bitfield permissions) | ✅ |
| Channel permission overrides | ✅ |
| Invite system (codes, max uses, expiry) | ✅ |
| File uploads (multipart, size limits) | ✅ |
| WebSocket gateway (real-time events) | ✅ |
| Typing indicators | ✅ |
| Presence (online/offline/idle/dnd) | ✅ |
| Full-text search (PostgreSQL tsvector) | ✅ |
| Message audit log (edit/delete tracking) | ✅ |
| Reactions | ✅ |
| Bridge schema (puppet users, entity mapping) | ✅ |

### Client — `rift-client` (Svelte 5)

| Feature | Status |
|---------|--------|
| Auth flow (login, register, token refresh) | ✅ |
| Server sidebar with icons | ✅ |
| Channel sidebar with categories | ✅ |
| Chat area with message history | ✅ |
| Message input with attachments | ✅ |
| Individual message rendering | ✅ |
| DM view | ✅ |
| Member list | ✅ |
| User settings | ✅ |
| Modal system | ✅ |
| WebSocket gateway (live updates) | ✅ |
| Reactive stores (auth, messages, servers, UI) | ✅ |

### Bridge — `rift-bridge` (Rust)

| Feature | Status |
|---------|--------|
| Discord Gateway v10 (zlib-stream, ETF) | ✅ |
| Heartbeat + resume + reconnect | ✅ |
| Full event dispatch (messages, reactions, typing, presence) | ✅ |
| Entity mapper (Discord ↔ Rift UUIDs) | ✅ |
| Puppet user creation | ✅ |
| Guild/channel/message sync | ✅ |
| Attachment bridging | ✅ |
| Reaction bridging | ✅ |
| Audit logging (edits/deletes) | ✅ |
| Bridge connection management | ✅ |
| Bidirectional relay (Rift → Discord) | 🔄 |
| DM bridging | 🔄 |
| Backfill (historical message import) | 🔄 |

---

## Architecture

```
rift/
├── server/                  # rift-server — Rust (Axum + SQLx + PostgreSQL)
│   ├── src/
│   │   ├── auth/            # JWT, Argon2, middleware
│   │   ├── db/              # Database connection + migrations
│   │   ├── models/          # User, Server, Channel, Message, Role, Permission
│   │   ├── routes/          # REST API handlers
│   │   └── ws/              # WebSocket gateway
│   └── migrations/          # PostgreSQL schema
│
├── client/                  # rift-client — Svelte 5 + TypeScript
│   └── src/
│       ├── lib/
│       │   ├── components/  # Auth, Chat, Sidebar, DM, Settings
│       │   ├── stores/      # Reactive state (auth, messages, servers, UI)
│       │   ├── api.ts       # HTTP client
│       │   ├── gateway.ts   # WebSocket client
│       │   └── types.ts     # Shared types
│       └── App.svelte       # Root component
│
└── bridge/                  # rift-bridge — Rust (Discord Gateway + mapper)
    └── src/
        ├── discord_types.rs # Discord API type definitions
        ├── gateway.rs       # Discord Gateway v10 WebSocket client
        └── mapper.rs        # Discord ↔ Rift entity mapper
```

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Server | Rust, Axum 0.8, Tokio, SQLx 0.8 |
| Database | PostgreSQL 16 (FTS, tsvector, UUID) |
| Auth | JWT (jsonwebtoken), Argon2 password hashing |
| Client | Svelte 5, TypeScript, Vite |
| Bridge | Rust, tokio-tungstenite, flate2 (zlib) |
| Real-time | WebSocket (server ↔ client), WebSocket (bridge ↔ Discord) |

### Data Model

```
users ──────────┐
  │              │
  ├── members ───┼── servers
  │   └── member_roles     │
  │       └── roles ───────┘
  │           └── channel_permissions
  │
  ├── messages ─── channels ─── servers
  │   ├── attachments
  │   └── reactions
  │
  ├── dm_participants ─── dm_channels
  │                        └── dm_messages
  │
  └── refresh_tokens

bridge_connections    # Discord ↔ Rift link config
message_audit_log     # Edit/delete history
```

Every table that participates in bridging has optional `bridge_source` + `bridge_remote_id` columns with partial unique indexes — zero overhead for native entities, instant lookup for bridged ones.

---

## Quick Start

### Prerequisites

- Rust 1.85+ (2024 edition)
- PostgreSQL 16+
- Node.js 22+ (for client build)

### 1. Database

```bash
sudo -u postgres createdb rift
sudo -u postgres psql rift < server/migrations/001_initial.sql
sudo -u postgres psql rift < server/migrations/002_bridge.sql
```

### 2. Server

```bash
cd server
cp .env.example .env
# Edit .env — set DATABASE_URL, JWT_SECRET

cargo run --release
# Listening on 0.0.0.0:3200
```

### 3. Client

```bash
cd client
npm install
npm run dev
# Dev server on http://localhost:5173
```

### 4. Bridge (optional)

```bash
cd bridge
DISCORD_TOKEN=your_token DATABASE_URL=postgres:///rift cargo run --release
```

---

## Discord Bridge

The bridge connects to Discord's Gateway as a user (selfbot-style) and mirrors everything into Rift's database. Your Discord friends see you on Discord — you see them in Rift.

### How It Works

1. **Connect** — Bridge authenticates to Discord Gateway v10 with your user token
2. **Sync** — Guilds, channels, and members are mirrored as Rift entities
3. **Puppet** — Each Discord user gets a local puppet account (marked `bridge_is_puppet = true`)
4. **Relay** — Messages flow bidirectionally:
   - Discord → Rift: stored locally with full content, attachments, reactions
   - Rift → Discord: sent via Discord API on your behalf
5. **Audit** — Edits and deletes are logged. Discord forgets — Rift remembers.

### What You Get

| Discord Limitation | Rift |
|-------------------|------|
| Message history requires scrolling | Full PostgreSQL search across all messages, all time |
| Edits replace original | Both versions preserved in audit log |
| Deletes are permanent | Original content preserved in audit log |
| 25MB upload limit (50MB Nitro) | Whatever your server can handle |
| Emoji restricted to server/Nitro | Any emoji, transcoded for delivery |
| No message export | `pg_dump` or API export — it's your database |
| Telemetry on everything | Zero telemetry. Your hardware, your data |
| Account banned = history gone | Local copy persists regardless |

### Bridge Schema

The bridge adds columns to existing tables rather than creating parallel structures:

```sql
-- Every bridgeable table gets these nullable columns:
bridge_source     VARCHAR(20)   -- 'discord' | NULL (native)
bridge_remote_id  VARCHAR(64)   -- Discord snowflake

-- Partial unique index: instant lookup, zero overhead for native rows
CREATE UNIQUE INDEX idx_users_bridge ON users(bridge_source, bridge_remote_id)
    WHERE bridge_source IS NOT NULL;
```

Additional bridge tables:

| Table | Purpose |
|-------|---------|
| `bridge_connections` | Active bridge configs (token, sync state, cursors) |
| `message_audit_log` | Edit/delete history with old + new content |
| `reactions` | Message reactions (new, needed for bridge) |

---

## API Reference

### Authentication

All endpoints require `Authorization: Bearer <jwt>` from login/register.

#### Auth

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/auth/register` | Create account |
| `POST` | `/api/auth/login` | Get JWT + refresh token |
| `POST` | `/api/auth/refresh` | Rotate refresh token |

#### Users

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/users/me` | Current user profile |
| `PATCH` | `/api/users/me` | Update profile |
| `PUT` | `/api/users/me/password` | Change password |
| `GET` | `/api/users/:id` | Get user by ID |

#### Servers

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/servers` | Create server |
| `GET` | `/api/servers` | List joined servers |
| `GET` | `/api/servers/:id` | Server details |
| `PATCH` | `/api/servers/:id` | Update server |
| `DELETE` | `/api/servers/:id` | Delete server |
| `GET` | `/api/servers/:id/members` | List members |
| `POST` | `/api/servers/:id/join` | Join via invite |
| `DELETE` | `/api/servers/:id/members/:uid` | Kick member |

#### Channels

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/servers/:id/channels` | Create channel |
| `GET` | `/api/servers/:id/channels` | List channels |
| `PATCH` | `/api/channels/:id` | Update channel |
| `DELETE` | `/api/channels/:id` | Delete channel |

#### Messages

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/channels/:id/messages` | Send message |
| `GET` | `/api/channels/:id/messages` | List messages (paginated) |
| `PATCH` | `/api/messages/:id` | Edit message |
| `DELETE` | `/api/messages/:id` | Delete message |

#### Direct Messages

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/dm/channels` | Create/get DM channel |
| `GET` | `/api/dm/channels` | List DM channels |
| `POST` | `/api/dm/channels/:id/messages` | Send DM |
| `GET` | `/api/dm/channels/:id/messages` | List DM messages |

#### Roles & Permissions

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/servers/:id/roles` | Create role |
| `GET` | `/api/servers/:id/roles` | List roles |
| `PATCH` | `/api/roles/:id` | Update role |
| `DELETE` | `/api/roles/:id` | Delete role |
| `POST` | `/api/roles/:id/members/:uid` | Assign role |
| `DELETE` | `/api/roles/:id/members/:uid` | Remove role |

#### Invites

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/servers/:id/invites` | Create invite |
| `GET` | `/api/servers/:id/invites` | List invites |
| `GET` | `/api/invites/:code` | Get invite details |

#### Uploads

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/upload` | Upload file (multipart) |

### WebSocket Gateway

Connect to `ws://host:3200/ws` with JWT in the first message.

#### Client → Server

```json
{ "type": "Identify", "token": "jwt_here" }
{ "type": "SendMessage", "channel_id": "uuid", "content": "hello" }
{ "type": "StartTyping", "channel_id": "uuid" }
{ "type": "UpdatePresence", "status": "online" }
{ "type": "SubscribeChannel", "channel_id": "uuid" }
{ "type": "SubscribeServer", "server_id": "uuid" }
```

#### Server → Client

```json
{ "type": "Ready", "data": { "user_id": "uuid", "username": "zan" } }
{ "type": "MessageCreate", "data": { "id": "uuid", "channel_id": "uuid", "content": "hello", ... } }
{ "type": "MessageUpdate", "data": { "id": "uuid", "content": "edited", ... } }
{ "type": "MessageDelete", "data": { "id": "uuid", "channel_id": "uuid" } }
{ "type": "TypingStart", "data": { "channel_id": "uuid", "user_id": "uuid" } }
{ "type": "PresenceUpdate", "data": { "user_id": "uuid", "status": "online" } }
```

---

## Permissions

Discord-style bitfield permissions with role hierarchy and channel overrides.

```rust
VIEW_CHANNELS    = 1 << 0
SEND_MESSAGES    = 1 << 1
READ_HISTORY     = 1 << 2
MANAGE_MESSAGES  = 1 << 3
ATTACH_FILES     = 1 << 4
MANAGE_CHANNELS  = 1 << 5
MANAGE_SERVER    = 1 << 6
MANAGE_ROLES     = 1 << 7
KICK_MEMBERS     = 1 << 8
BAN_MEMBERS      = 1 << 9
CREATE_INVITES   = 1 << 10
MANAGE_INVITES   = 1 << 11
MENTION_EVERYONE = 1 << 12
ADMINISTRATOR    = 1 << 30  // Bypasses all checks
```

Channel permission overrides can allow or deny specific permissions per-role or per-user, exactly like Discord.

---

## Self-Hosting

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL connection string |
| `JWT_SECRET` | — | Secret for signing JWTs |
| `LISTEN_ADDR` | `0.0.0.0:3200` | Server bind address |
| `UPLOAD_DIR` | `./uploads` | File upload storage path |
| `MAX_UPLOAD_BYTES` | `26214400` | Max upload size (25MB) |
| `DISCORD_TOKEN` | — | Discord user token (bridge only) |
| `RUST_LOG` | `rift_server=debug` | Log level filter |

### Reverse Proxy

```nginx
server {
    server_name chat.example.com;

    location / {
        proxy_pass http://127.0.0.1:3200;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Database

PostgreSQL 16+ recommended. The schema uses:
- `gen_random_uuid()` for primary keys
- `tsvector` + GIN indexes for full-text search
- Partial unique indexes for bridge entity lookups
- `TIMESTAMPTZ` everywhere (UTC)

---

## Roadmap

- [ ] Voice channels (WebRTC)
- [ ] Thread support
- [ ] Message search UI
- [ ] E2E encryption (optional per-channel)
- [ ] Federation (Rift ↔ Rift)
- [ ] Mobile client (Tauri or React Native)
- [ ] Plugin system
- [ ] Custom emoji/sticker hosting
- [ ] Webhook integrations
- [ ] Bot API (Discord-compatible subset)

---

## Stats

| Component | Language | Lines |
|-----------|----------|-------|
| Server | Rust | ~3,500 |
| Client | Svelte/TS | ~5,500 |
| Bridge | Rust | ~1,000 |
| Migrations | SQL | ~275 |
| **Total** | | **~10,300** |

---

## License

MIT
