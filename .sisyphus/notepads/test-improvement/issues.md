# Chat History Functionality Issues

## Issue: Missing Chat History Implementation

**Date**: Tue Mar 10 2026

**Description**: 
The database schema includes a `chat_history` table (defined in `migrations/20260305000000_init.sql`), but there is no actual implementation that uses this table for persistent chat history storage.

**Current State**:
- ✅ `chat_history` table exists in database schema
- ✅ Table has proper structure with indexes (`room_id`, `created_at`)
- ❌ No methods in `Database` struct to interact with `chat_history`
- ❌ Conversation history is managed entirely in memory via `ConversationManager`
- ❌ No integration between conversation management and database persistence

**Evidence**:
- `src/store/database.rs` contains only connection and migration logic, no chat history methods
- `src/conversation.rs` manages all conversation state in memory using `HashMap<String, Vec<ChatCompletionRequestMessage>>`
- `src/ai_service.rs` uses only the in-memory `ConversationManager`
- No SQL queries found that interact with `chat_history` table
- Grep search confirms no usage of `chat_history` outside of schema definition and test verification

**Impact**:
- Chat history is lost when the bot restarts
- No persistent conversation history across sessions
- Database table is unused (wasted storage)

**Recommendation**:
Implement chat history persistence by:
1. Adding methods to `Database` struct for saving/loading chat history
2. Integrating database persistence into `ConversationManager` 
3. Creating comprehensive tests for chat history functionality