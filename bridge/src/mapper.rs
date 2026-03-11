use sqlx::PgPool;
use uuid::Uuid;

use crate::discord_types::*;

/// Maps Discord snowflake IDs to Rift UUIDs, creating entities as needed.
pub struct BridgeMapper {
    pool: PgPool,
    /// The local Rift user ID that owns this bridge connection
    local_user_id: Uuid,
    /// The Discord user ID we're connected as
    discord_user_id: String,
}

impl BridgeMapper {
    pub fn new(pool: PgPool, local_user_id: Uuid, discord_user_id: String) -> Self {
        Self { pool, local_user_id, discord_user_id }
    }

    /// Is this message from the bridged user (i.e., from us)?
    pub fn is_self(&self, discord_user_id: &str) -> bool {
        self.discord_user_id == discord_user_id
    }

    // ─── User Puppets ───

    /// Get or create a local puppet user for a Discord user.
    /// If the Discord user is us, return our real local user ID.
    pub async fn resolve_user(&self, user: &DiscordUser) -> Result<Uuid, sqlx::Error> {
        if self.is_self(&user.id) {
            return Ok(self.local_user_id);
        }

        // Check if puppet exists
        if let Some(id) = self.get_bridged_user(&user.id).await? {
            // Update avatar/display name if changed
            self.update_puppet(id, user).await?;
            return Ok(id);
        }

        // Create puppet
        let puppet_username = format!("d_{}", &user.id);
        let display_name = user.display_name();
        let avatar_url = user.avatar_url();

        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO users (username, email, password_hash, display_name, avatar_url,
                                  bridge_source, bridge_remote_id, bridge_is_puppet)
               VALUES ($1, $2, 'PUPPET_NO_LOGIN', $3, $4, 'discord', $5, true)
               ON CONFLICT (bridge_source, bridge_remote_id) WHERE bridge_source IS NOT NULL
               DO UPDATE SET display_name = EXCLUDED.display_name,
                             avatar_url = EXCLUDED.avatar_url,
                             updated_at = NOW()
               RETURNING id"#,
        )
        .bind(&puppet_username)
        .bind(format!("puppet_{}@bridge.local", user.id))
        .bind(display_name)
        .bind(&avatar_url)
        .bind(&user.id)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created puppet user for Discord {} ({})", user.username, user.id);
        Ok(row.0)
    }

    async fn get_bridged_user(&self, discord_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE bridge_source = 'discord' AND bridge_remote_id = $1",
        )
        .bind(discord_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    async fn update_puppet(&self, user_id: Uuid, user: &DiscordUser) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE users SET display_name = $2, avatar_url = $3, updated_at = NOW()
               WHERE id = $1 AND bridge_is_puppet = true"#,
        )
        .bind(user_id)
        .bind(user.display_name())
        .bind(&user.avatar_url())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ─── Servers (Guilds) ───

    /// Get or create a local server for a Discord guild.
    pub async fn resolve_guild(&self, guild: &DiscordGuild) -> Result<Uuid, sqlx::Error> {
        if let Some(id) = self.get_bridged_server(&guild.id).await? {
            // Update name/description
            sqlx::query(
                "UPDATE servers SET name = $2, description = $3 WHERE id = $1"
            )
            .bind(id)
            .bind(&guild.name)
            .bind(&guild.description)
            .execute(&self.pool)
            .await?;
            return Ok(id);
        }

        // Create server owned by the bridge user
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO servers (name, description, owner_id, bridge_source, bridge_remote_id)
               VALUES ($1, $2, $3, 'discord', $4)
               ON CONFLICT (bridge_source, bridge_remote_id) WHERE bridge_source IS NOT NULL
               DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description
               RETURNING id"#,
        )
        .bind(&guild.name)
        .bind(&guild.description)
        .bind(self.local_user_id)
        .bind(&guild.id)
        .fetch_one(&self.pool)
        .await?;

        // Add bridge user as member
        sqlx::query(
            "INSERT INTO members (server_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(row.0)
        .bind(self.local_user_id)
        .execute(&self.pool)
        .await?;

        // Create @everyone role
        sqlx::query(
            r#"INSERT INTO roles (server_id, name, permissions, position, is_default)
               VALUES ($1, '@everyone', $2, 0, true)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(row.0)
        .bind(crate::PERMS_DEFAULT)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created bridged server: {} ({})", guild.name, guild.id);
        Ok(row.0)
    }

    pub async fn get_bridged_server(&self, discord_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM servers WHERE bridge_source = 'discord' AND bridge_remote_id = $1",
        )
        .bind(discord_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    // ─── Channels ───

    pub async fn resolve_channel(
        &self,
        channel: &DiscordChannel,
        server_id: Uuid,
    ) -> Result<Uuid, sqlx::Error> {
        if let Some(id) = self.get_bridged_channel(&channel.id).await? {
            sqlx::query(
                "UPDATE channels SET name = COALESCE($2, name), topic = COALESCE($3, topic) WHERE id = $1"
            )
            .bind(id)
            .bind(&channel.name)
            .bind(&channel.topic)
            .execute(&self.pool)
            .await?;
            return Ok(id);
        }

        let chan_type = if channel.is_text() { "text" } else { "voice" };
        let name = channel.name.as_deref().unwrap_or("unknown");
        let position = channel.position.unwrap_or(0);

        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO channels (server_id, name, topic, channel_type, position,
                                     bridge_source, bridge_remote_id)
               VALUES ($1, $2, $3, $4, $5, 'discord', $6)
               ON CONFLICT (bridge_source, bridge_remote_id) WHERE bridge_source IS NOT NULL
               DO UPDATE SET name = EXCLUDED.name, topic = EXCLUDED.topic, position = EXCLUDED.position
               RETURNING id"#,
        )
        .bind(server_id)
        .bind(name)
        .bind(&channel.topic)
        .bind(chan_type)
        .bind(position)
        .bind(&channel.id)
        .fetch_one(&self.pool)
        .await?;

        tracing::debug!("Resolved channel: #{name} ({})", channel.id);
        Ok(row.0)
    }

    pub async fn get_bridged_channel(&self, discord_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM channels WHERE bridge_source = 'discord' AND bridge_remote_id = $1",
        )
        .bind(discord_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    // ─── Messages ───

    /// Store a Discord message in the local database.
    pub async fn store_message(
        &self,
        msg: &DiscordMessage,
        channel_id: Uuid,
        author_id: Uuid,
    ) -> Result<Uuid, sqlx::Error> {
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO messages (channel_id, author_id, content, bridge_source, bridge_remote_id)
               VALUES ($1, $2, $3, 'discord', $4)
               ON CONFLICT (bridge_source, bridge_remote_id) WHERE bridge_source IS NOT NULL
               DO UPDATE SET content = EXCLUDED.content
               RETURNING id"#,
        )
        .bind(channel_id)
        .bind(author_id)
        .bind(&msg.content)
        .bind(&msg.id)
        .fetch_one(&self.pool)
        .await?;

        // Store attachments
        for att in &msg.attachments {
            sqlx::query(
                r#"INSERT INTO attachments (message_id, filename, url, content_type, size_bytes,
                                            bridge_source, bridge_remote_id)
                   VALUES ($1, $2, $3, $4, $5, 'discord', $6)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(row.0)
            .bind(&att.filename)
            .bind(&att.url)
            .bind(&att.content_type)
            .bind(att.size.map(|s| s as i64))
            .bind(&att.id)
            .execute(&self.pool)
            .await?;
        }

        Ok(row.0)
    }

    /// Get the local message ID for a Discord message ID
    pub async fn get_bridged_message(&self, discord_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM messages WHERE bridge_source = 'discord' AND bridge_remote_id = $1",
        )
        .bind(discord_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Get Discord message ID for a local message
    pub async fn get_discord_message_id(&self, local_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT bridge_remote_id FROM messages WHERE id = $1 AND bridge_source = 'discord'",
        )
        .bind(local_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Get Discord channel ID for a local channel
    pub async fn get_discord_channel_id(&self, local_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT bridge_remote_id FROM channels WHERE id = $1 AND bridge_source = 'discord'",
        )
        .bind(local_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    // ─── Audit Log ───

    /// Log a message edit or delete (Discord hides these — we don't)
    pub async fn log_audit(
        &self,
        message_id: Uuid,
        channel_id: Uuid,
        action: &str,
        old_content: Option<&str>,
        new_content: Option<&str>,
        actor_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO message_audit_log (message_id, channel_id, action, old_content, new_content, actor_id, bridge_source)
               VALUES ($1, $2, $3, $4, $5, $6, 'discord')"#,
        )
        .bind(message_id)
        .bind(channel_id)
        .bind(action)
        .bind(old_content)
        .bind(new_content)
        .bind(actor_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ─── Members ───

    /// Add a puppet user as a member of a bridged server
    pub async fn add_member(&self, server_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO members (server_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(server_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_member(&self, server_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM members WHERE server_id = $1 AND user_id = $2"
        )
        .bind(server_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ─── Reactions ───

    pub async fn add_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO reactions (message_id, user_id, emoji, bridge_source)
               VALUES ($1, $2, $3, 'discord')
               ON CONFLICT (message_id, user_id, emoji) DO NOTHING"#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3"
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
