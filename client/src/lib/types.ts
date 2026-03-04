// ── Types matching the Rust backend models ──

export interface User {
  id: string;
  username: string;
  display_name: string | null;
  email: string;
  avatar_url: string | null;
  status: string;
  about: string | null;
  created_at: string;
  updated_at: string;
}

export interface PublicUser {
  id: string;
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  status: string;
  about: string | null;
}

export interface AuthResponse {
  token: string;
  user: PublicUser;
}

export interface Server {
  id: string;
  name: string;
  icon_url: string | null;
  description: string | null;
  owner_id: string;
  created_at: string;
}

export interface ServerWithDetails {
  server: Server;
  channels: Channel[];
  roles: Role[];
}

export interface Channel {
  id: string;
  server_id: string;
  name: string;
  topic: string | null;
  channel_type: string;
  position: number;
  created_at: string;
}

export interface Message {
  id: string;
  channel_id: string;
  author_id: string;
  content: string;
  edited_at: string | null;
  created_at: string;
}

export interface MessageWithAuthor {
  id: string;
  channel_id: string;
  author_id: string;
  content: string;
  edited_at: string | null;
  created_at: string;
  author_username: string;
  author_display_name: string | null;
  author_avatar_url: string | null;
}

export interface Member {
  server_id: string;
  user_id: string;
  nickname: string | null;
  joined_at: string;
}

export interface MemberWithUser extends Member {
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  status: string;
}

export interface Role {
  id: string;
  server_id: string;
  name: string;
  color: number;
  permissions: number;
  position: number;
  is_default: boolean;
  created_at: string;
}

export interface Invite {
  code: string;
  server_id: string;
  creator_id: string;
  max_uses: number | null;
  uses: number;
  expires_at: string | null;
  created_at: string;
}

export interface DmChannel {
  id: string;
  created_at: string;
  other_user_id: string;
  other_username: string;
  other_display_name: string | null;
  other_avatar_url: string | null;
}

export interface DmMessage {
  id: string;
  dm_channel_id: string;
  sender_id: string;
  content: string;
  created_at: string;
}

// ── Permissions bitfield (mirrors server/src/models/permissions.rs) ──

export const Permissions = {
  VIEW_CHANNELS:    1 << 0,
  SEND_MESSAGES:    1 << 1,
  READ_HISTORY:     1 << 2,
  MANAGE_MESSAGES:  1 << 3,
  ATTACH_FILES:     1 << 4,
  MANAGE_CHANNELS:  1 << 5,
  MANAGE_SERVER:    1 << 6,
  MANAGE_ROLES:     1 << 7,
  KICK_MEMBERS:     1 << 8,
  BAN_MEMBERS:      1 << 9,
  CREATE_INVITES:   1 << 10,
  MANAGE_INVITES:   1 << 11,
  MENTION_EVERYONE: 1 << 12,
  ADMINISTRATOR:    1 << 30,
} as const;

export function hasPermission(permissions: number, permission: number): boolean {
  return (permissions & Permissions.ADMINISTRATOR) !== 0 || (permissions & permission) !== 0;
}

// ── Gateway event types (mirrors server/src/ws/gateway.rs) ──

export type GatewayEvent =
  | { type: 'Ready'; data: { user_id: string; username: string } }
  | { type: 'MessageCreate'; data: {
      id: string; channel_id: string; author_id: string;
      author_username: string; author_display_name: string | null;
      author_avatar_url: string | null; content: string; created_at: string;
    }}
  | { type: 'MessageUpdate'; data: { id: string; channel_id: string; content: string; edited_at: string } }
  | { type: 'MessageDelete'; data: { id: string; channel_id: string } }
  | { type: 'TypingStart'; data: { channel_id: string; user_id: string; username: string } }
  | { type: 'PresenceUpdate'; data: { user_id: string; status: string } }
  | { type: 'MemberJoin'; data: { server_id: string; user_id: string; username: string } }
  | { type: 'MemberLeave'; data: { server_id: string; user_id: string } }
  | { type: 'ChannelCreate'; data: { id: string; server_id: string; name: string; channel_type: string } }
  | { type: 'ChannelDelete'; data: { id: string; server_id: string } };

export type GatewayCommand =
  | { type: 'Identify'; data: { token: string } }
  | { type: 'Typing'; data: { channel_id: string } }
  | { type: 'UpdatePresence'; data: { status: string } }
  | { type: 'Subscribe'; data: { server_ids: string[] } };
