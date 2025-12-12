# WebSocket Gateway Protocol Specification

## Discord-like Chat Server - Real-time Communication Protocol

This document defines the WebSocket Gateway protocol for real-time communication between clients and the server.

---

## Table of Contents

1. [Overview](#overview)
2. [Connection](#connection)
3. [Message Format](#message-format)
4. [Op Codes](#op-codes)
5. [Gateway Events](#gateway-events)
6. [Connection Lifecycle](#connection-lifecycle)
7. [Heartbeat System](#heartbeat-system)
8. [Session Management](#session-management)
9. [Error Handling](#error-handling)
10. [Rate Limiting](#rate-limiting)

---

## Overview

The Gateway is a WebSocket-based API that enables real-time bidirectional communication between clients and the server.

### Key Features

- **Real-time events**: Messages, presence updates, typing indicators
- **Heartbeat system**: Keep-alive mechanism with automatic reconnection
- **Session resume**: Reconnect without losing events (2-minute window)
- **Sequence numbers**: Track events for resume capability

### Gateway URL

```
Development: ws://localhost:8081/gateway
Production:  wss://api.example.com/gateway
```

---

## Connection

### Establishing Connection

1. Client opens WebSocket connection to gateway URL
2. Server sends `Hello` (op 10) with heartbeat interval
3. Client sends `Identify` (op 2) with authentication token
4. Server sends `Ready` (op 0) with initial state
5. Server sends `Guild Create` (op 0) for each guild

### Connection Headers

```
Authorization: Bearer <access_token>  (optional, can use Identify instead)
```

---

## Message Format

All messages follow a consistent JSON structure:

```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 42,
  "d": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `op` | integer | Operation code (see Op Codes) |
| `t` | string? | Event name (only for op=0 Dispatch) |
| `s` | integer? | Sequence number (only for op=0 Dispatch) |
| `d` | object? | Event data payload |

---

## Op Codes

| Op | Name | Client | Server | Description |
|----|------|--------|--------|-------------|
| 0 | Dispatch | ❌ | ✅ | Server dispatches an event |
| 1 | Heartbeat | ✅ | ✅ | Keep connection alive |
| 2 | Identify | ✅ | ❌ | Authenticate session |
| 3 | Presence Update | ✅ | ❌ | Update online status |
| 4 | Resume | ✅ | ❌ | Resume dropped connection |
| 5 | Reconnect | ❌ | ✅ | Server requests client reconnect |
| 7 | Invalid Session | ❌ | ✅ | Session is invalid |
| 10 | Hello | ❌ | ✅ | Sent on connect |
| 11 | Heartbeat ACK | ❌ | ✅ | Heartbeat acknowledged |

### Op 0: Dispatch

Server sends events to client.

```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 42,
  "d": {
    "id": "1234567890123456789",
    "channel_id": "9876543210987654321",
    "author": { ... },
    "content": "Hello, World!"
  }
}
```

### Op 1: Heartbeat

Client sends to keep connection alive. Server responds with Heartbeat ACK.

**Client → Server:**
```json
{
  "op": 1,
  "d": 41
}
```

The `d` field contains the last received sequence number (or `null` if none received).

### Op 2: Identify

Client authenticates after receiving Hello.

**Client → Server:**
```json
{
  "op": 2,
  "d": {
    "token": "Bearer eyJhbGciOiJIUzI1NiIs...",
    "properties": {
      "os": "windows",
      "browser": "python-client",
      "device": "desktop"
    }
  }
}
```

### Op 3: Presence Update

Client updates their online status.

**Client → Server:**
```json
{
  "op": 3,
  "d": {
    "status": "idle"
  }
}
```

**Valid statuses:** `online`, `idle`, `dnd`, `offline`

### Op 4: Resume

Client attempts to resume a disconnected session.

**Client → Server:**
```json
{
  "op": 4,
  "d": {
    "token": "Bearer eyJhbGciOiJIUzI1NiIs...",
    "session_id": "abc123def456",
    "seq": 41
  }
}
```

### Op 5: Reconnect

Server requests client to reconnect.

**Server → Client:**
```json
{
  "op": 5,
  "d": null
}
```

Client should:
1. Close current connection
2. Reconnect to gateway
3. Attempt Resume (op 4)

### Op 7: Invalid Session

Session is invalid. Client must re-identify.

**Server → Client:**
```json
{
  "op": 7,
  "d": false
}
```

The `d` field indicates if the session is resumable:
- `true`: Can attempt Resume
- `false`: Must send new Identify

### Op 10: Hello

Sent immediately after connection.

**Server → Client:**
```json
{
  "op": 10,
  "d": {
    "heartbeat_interval": 45000
  }
}
```

Client MUST start sending Heartbeats at this interval.

### Op 11: Heartbeat ACK

Server acknowledges heartbeat.

**Server → Client:**
```json
{
  "op": 11,
  "d": null
}
```

If client doesn't receive ACK before next heartbeat is due, consider connection dead.

---

## Gateway Events

Events are dispatched via op=0 with event name in `t` field.

### Connection Events

#### READY

Sent after successful Identify.

```json
{
  "op": 0,
  "t": "READY",
  "s": 1,
  "d": {
    "v": 1,
    "user": {
      "id": "1234567890123456789",
      "username": "testuser",
      "discriminator": "0001",
      "avatar": "abc123",
      "bot": false
    },
    "guilds": [
      { "id": "111222333444555666", "unavailable": true },
      { "id": "777888999000111222", "unavailable": true }
    ],
    "session_id": "abc123def456ghi789",
    "resume_gateway_url": "wss://gateway.example.com"
  }
}
```

#### RESUMED

Sent after successful Resume.

```json
{
  "op": 0,
  "t": "RESUMED",
  "s": 50,
  "d": {}
}
```

After RESUMED, server replays all missed events.

---

### Guild Events

#### GUILD_CREATE

Sent for each guild on connect, or when joining a new guild.

```json
{
  "op": 0,
  "t": "GUILD_CREATE",
  "s": 2,
  "d": {
    "id": "111222333444555666",
    "name": "My Server",
    "icon": "abc123",
    "description": "A cool server",
    "owner_id": "1234567890123456789",
    "channels": [
      {
        "id": "222333444555666777",
        "name": "general",
        "type": 0,
        "position": 0,
        "topic": "General discussion"
      }
    ],
    "roles": [
      {
        "id": "111222333444555666",
        "name": "@everyone",
        "permissions": "104324673",
        "position": 0
      }
    ],
    "members": [
      {
        "user": { "id": "1234567890123456789", "username": "testuser" },
        "nickname": null,
        "roles": [],
        "joined_at": "2024-01-15T10:30:00Z"
      }
    ],
    "member_count": 42,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

#### GUILD_UPDATE

Sent when guild settings change.

```json
{
  "op": 0,
  "t": "GUILD_UPDATE",
  "s": 15,
  "d": {
    "id": "111222333444555666",
    "name": "Renamed Server",
    "icon": "def456",
    "description": "Updated description",
    "owner_id": "1234567890123456789"
  }
}
```

#### GUILD_DELETE

Sent when leaving, kicked from, or guild is deleted.

```json
{
  "op": 0,
  "t": "GUILD_DELETE",
  "s": 20,
  "d": {
    "id": "111222333444555666",
    "unavailable": false
  }
}
```

- `unavailable: true` = Guild outage (temporary)
- `unavailable: false` = Left/kicked/deleted (permanent)

---

### Channel Events

#### CHANNEL_CREATE

```json
{
  "op": 0,
  "t": "CHANNEL_CREATE",
  "s": 25,
  "d": {
    "id": "333444555666777888",
    "guild_id": "111222333444555666",
    "name": "new-channel",
    "type": 0,
    "position": 5,
    "topic": null,
    "parent_id": "222333444555666777"
  }
}
```

#### CHANNEL_UPDATE

```json
{
  "op": 0,
  "t": "CHANNEL_UPDATE",
  "s": 26,
  "d": {
    "id": "333444555666777888",
    "guild_id": "111222333444555666",
    "name": "renamed-channel",
    "type": 0,
    "position": 3,
    "topic": "New topic"
  }
}
```

#### CHANNEL_DELETE

```json
{
  "op": 0,
  "t": "CHANNEL_DELETE",
  "s": 27,
  "d": {
    "id": "333444555666777888",
    "guild_id": "111222333444555666",
    "type": 0
  }
}
```

---

### Message Events

#### MESSAGE_CREATE

```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 30,
  "d": {
    "id": "444555666777888999",
    "channel_id": "333444555666777888",
    "guild_id": "111222333444555666",
    "author": {
      "id": "1234567890123456789",
      "username": "testuser",
      "discriminator": "0001",
      "avatar": "abc123",
      "bot": false
    },
    "content": "Hello everyone!",
    "timestamp": "2024-01-15T10:30:00Z",
    "edited_timestamp": null,
    "attachments": [],
    "reactions": [],
    "message_reference": null
  }
}
```

**With Reply:**
```json
{
  "d": {
    "id": "555666777888999000",
    "content": "I agree!",
    "message_reference": {
      "message_id": "444555666777888999",
      "channel_id": "333444555666777888",
      "guild_id": "111222333444555666"
    },
    "referenced_message": {
      "id": "444555666777888999",
      "content": "Hello everyone!",
      "author": { ... }
    }
  }
}
```

#### MESSAGE_UPDATE

```json
{
  "op": 0,
  "t": "MESSAGE_UPDATE",
  "s": 31,
  "d": {
    "id": "444555666777888999",
    "channel_id": "333444555666777888",
    "guild_id": "111222333444555666",
    "content": "Hello everyone! (edited)",
    "edited_timestamp": "2024-01-15T10:35:00Z"
  }
}
```

Note: Partial update - only changed fields included.

#### MESSAGE_DELETE

```json
{
  "op": 0,
  "t": "MESSAGE_DELETE",
  "s": 32,
  "d": {
    "id": "444555666777888999",
    "channel_id": "333444555666777888",
    "guild_id": "111222333444555666"
  }
}
```

---

### Reaction Events

#### MESSAGE_REACTION_ADD

```json
{
  "op": 0,
  "t": "MESSAGE_REACTION_ADD",
  "s": 35,
  "d": {
    "user_id": "1234567890123456789",
    "channel_id": "333444555666777888",
    "message_id": "444555666777888999",
    "guild_id": "111222333444555666",
    "emoji": "👍"
  }
}
```

#### MESSAGE_REACTION_REMOVE

```json
{
  "op": 0,
  "t": "MESSAGE_REACTION_REMOVE",
  "s": 36,
  "d": {
    "user_id": "1234567890123456789",
    "channel_id": "333444555666777888",
    "message_id": "444555666777888999",
    "guild_id": "111222333444555666",
    "emoji": "👍"
  }
}
```

---

### Member Events

#### GUILD_MEMBER_ADD

```json
{
  "op": 0,
  "t": "GUILD_MEMBER_ADD",
  "s": 40,
  "d": {
    "guild_id": "111222333444555666",
    "user": {
      "id": "9876543210987654321",
      "username": "newuser",
      "discriminator": "0042",
      "avatar": null
    },
    "nickname": null,
    "roles": [],
    "joined_at": "2024-01-15T11:00:00Z"
  }
}
```

#### GUILD_MEMBER_UPDATE

```json
{
  "op": 0,
  "t": "GUILD_MEMBER_UPDATE",
  "s": 41,
  "d": {
    "guild_id": "111222333444555666",
    "user": { "id": "9876543210987654321" },
    "nickname": "New Nickname",
    "roles": ["role_id_1", "role_id_2"]
  }
}
```

#### GUILD_MEMBER_REMOVE

```json
{
  "op": 0,
  "t": "GUILD_MEMBER_REMOVE",
  "s": 42,
  "d": {
    "guild_id": "111222333444555666",
    "user": {
      "id": "9876543210987654321",
      "username": "leftuser"
    }
  }
}
```

---

### Presence Events

#### PRESENCE_UPDATE

```json
{
  "op": 0,
  "t": "PRESENCE_UPDATE",
  "s": 45,
  "d": {
    "user": {
      "id": "1234567890123456789"
    },
    "guild_id": "111222333444555666",
    "status": "online"
  }
}
```

**Statuses:**
- `online` - User is online
- `idle` - User is idle (away)
- `dnd` - Do not disturb
- `offline` - User is offline

#### TYPING_START

```json
{
  "op": 0,
  "t": "TYPING_START",
  "s": 46,
  "d": {
    "channel_id": "333444555666777888",
    "guild_id": "111222333444555666",
    "user_id": "1234567890123456789",
    "timestamp": 1705315800
  }
}
```

Typing indicator expires after 10 seconds.

---

### User Events

#### USER_UPDATE

Sent when the current user's settings change.

```json
{
  "op": 0,
  "t": "USER_UPDATE",
  "s": 50,
  "d": {
    "id": "1234567890123456789",
    "username": "newusername",
    "discriminator": "0001",
    "avatar": "newhash123"
  }
}
```

---

## Connection Lifecycle

### Normal Connection Flow

```
Client                                    Server
   |                                         |
   |-------- WebSocket Connect ------------->|
   |                                         |
   |<------------- Hello (op=10) ------------|
   |         {"heartbeat_interval": 45000}   |
   |                                         |
   |------------ Identify (op=2) ----------->|
   |         {"token": "..."}                |
   |                                         |
   |<------------- Ready (op=0) -------------|
   |         {user, guilds[], session_id}    |
   |                                         |
   |<-------- Guild Create (op=0) -----------|
   |         (full guild data)               |
   |<-------- Guild Create (op=0) -----------|
   |         (for each guild)                |
   |                                         |
   |----------- Heartbeat (op=1) ----------->|
   |<-------- Heartbeat ACK (op=11) ---------|
   |                                         |
   |<-------- Message Create (op=0) ---------|
   |         (real-time events)              |
   |                                         |
```

### Resume Flow (After Disconnect)

```
Client                                    Server
   |                                         |
   |-------- Connection Lost ----------------|
   |                                         |
   |-------- WebSocket Reconnect ----------->|
   |                                         |
   |<------------- Hello (op=10) ------------|
   |                                         |
   |------------ Resume (op=4) ------------->|
   |    {token, session_id, seq}             |
   |                                         |
   |<-------- Missed Event (op=0) -----------|
   |<-------- Missed Event (op=0) -----------|
   |<-------- Missed Event (op=0) -----------|
   |<---------- Resumed (op=0) --------------|
   |                                         |
```

### Invalid Session Flow

```
Client                                    Server
   |                                         |
   |------------ Resume (op=4) ------------->|
   |                                         |
   |<------ Invalid Session (op=7) ----------|
   |         {resumable: false}              |
   |                                         |
   |------------ Identify (op=2) ----------->|
   |                                         |
   |<------------- Ready (op=0) -------------|
   |                                         |
```

---

## Heartbeat System

### Heartbeat Interval

- Server specifies interval in Hello (typically 45000ms)
- Client should add jitter: `interval * random(0.9, 1.1)`
- First heartbeat sent after: `interval * random(0, 1)`

### Heartbeat Protocol

1. Client sends Heartbeat (op=1) with last sequence number
2. Server responds with Heartbeat ACK (op=11)
3. If no ACK received before next heartbeat due:
   - Consider connection zombied
   - Close connection
   - Attempt reconnect + resume

### Implementation

```python
import asyncio
import random

class HeartbeatManager:
    def __init__(self, interval_ms: int):
        self.interval = interval_ms / 1000
        self.last_ack = True
        self.sequence = None

    async def start(self, ws):
        # Random first heartbeat
        await asyncio.sleep(self.interval * random.random())

        while True:
            if not self.last_ack:
                # Zombied connection
                await ws.close(code=1000)
                return

            self.last_ack = False
            await ws.send(json.dumps({"op": 1, "d": self.sequence}))

            # Add jitter
            jittered = self.interval * random.uniform(0.9, 1.1)
            await asyncio.sleep(jittered)

    def ack(self):
        self.last_ack = True

    def update_sequence(self, seq: int):
        self.sequence = seq
```

---

## Session Management

### Session Properties

| Property | Type | Description |
|----------|------|-------------|
| `session_id` | string | Unique session identifier |
| `user_id` | snowflake | Authenticated user |
| `sequence` | integer | Last event sequence |
| `guilds` | snowflake[] | Subscribed guilds |
| `created_at` | timestamp | Session start time |
| `resume_url` | string | Gateway URL for resume |

### Session Storage (Redis)

```
Key: ws_session:{session_id}
TTL: 120 seconds (2 minutes after disconnect)

Value: {
  "user_id": "1234567890123456789",
  "sequence": 42,
  "guilds": ["guild_id_1", "guild_id_2"],
  "created_at": 1705315800,
  "resume_url": "wss://gateway.example.com"
}
```

### Event Queue (for Resume)

```
Key: ws_events:{session_id}
Type: List (LPUSH/RPOP)
TTL: 120 seconds

Items: JSON-encoded events
Max Length: 1000 events
```

---

## Error Handling

### Close Codes

| Code | Name | Description | Reconnect? |
|------|------|-------------|------------|
| 4000 | Unknown Error | Unknown error occurred | Yes |
| 4001 | Unknown Opcode | Invalid opcode sent | Yes |
| 4002 | Decode Error | Invalid payload encoding | Yes |
| 4003 | Not Authenticated | Sent payload before Identify | No |
| 4004 | Authentication Failed | Invalid token | No |
| 4005 | Already Authenticated | Sent Identify twice | Yes |
| 4007 | Invalid Sequence | Invalid sequence for Resume | Yes |
| 4008 | Rate Limited | Too many requests | Yes (after delay) |
| 4009 | Session Timeout | Session expired | Yes |
| 4010 | Invalid Shard | Invalid shard configuration | No |
| 4011 | Sharding Required | Must use sharding | No |
| 4012 | Invalid API Version | Outdated API version | No |

### Reconnection Strategy

```python
async def connect_with_retry(self):
    attempts = 0
    max_attempts = 5

    while attempts < max_attempts:
        try:
            await self.connect()
            attempts = 0  # Reset on success
        except ConnectionClosed as e:
            if not self.should_reconnect(e.code):
                raise

            attempts += 1
            delay = min(2 ** attempts, 60)  # Exponential backoff, max 60s
            await asyncio.sleep(delay)

def should_reconnect(self, code: int) -> bool:
    non_reconnectable = {4003, 4004, 4010, 4011, 4012}
    return code not in non_reconnectable
```

---

## Rate Limiting

### Limits

| Action | Limit |
|--------|-------|
| Identify | 1 per 5 seconds |
| Heartbeat | 1 per heartbeat_interval |
| Presence Update | 5 per 60 seconds |
| General payloads | 120 per 60 seconds |

### Rate Limit Response

Connection closed with code 4008 (Rate Limited).

Client should:
1. Wait for reconnect delay (included in close frame)
2. Reconnect
3. Resume session

---

## Appendix: Complete Event List

| Event | Description |
|-------|-------------|
| `READY` | Connection established, contains initial state |
| `RESUMED` | Session successfully resumed |
| `GUILD_CREATE` | Guild available, joined, or created |
| `GUILD_UPDATE` | Guild settings changed |
| `GUILD_DELETE` | Left guild, kicked, or guild deleted |
| `CHANNEL_CREATE` | Channel created |
| `CHANNEL_UPDATE` | Channel updated |
| `CHANNEL_DELETE` | Channel deleted |
| `MESSAGE_CREATE` | New message |
| `MESSAGE_UPDATE` | Message edited |
| `MESSAGE_DELETE` | Message deleted |
| `MESSAGE_REACTION_ADD` | Reaction added |
| `MESSAGE_REACTION_REMOVE` | Reaction removed |
| `GUILD_MEMBER_ADD` | User joined guild |
| `GUILD_MEMBER_UPDATE` | Member updated (roles, nickname) |
| `GUILD_MEMBER_REMOVE` | User left guild |
| `PRESENCE_UPDATE` | User status changed |
| `TYPING_START` | User started typing |
| `USER_UPDATE` | Current user updated |

---

*Document Version: 1.0.0*
*Last Updated: 2024*
