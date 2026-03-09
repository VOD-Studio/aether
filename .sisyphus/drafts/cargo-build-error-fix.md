# Draft: Cargo Build Error Analysis & Fix

## Requirements (confirmed)
- Analyze and fix cargo build errors in the Aether Matrix Bot project
- Identify root cause of build failures
- Provide actionable fix plan

## Technical Decisions
- (Pending)

## Research Findings

### Build Error Analysis (cargo build output)
Total errors: 38, 1 warning. Primary error categories:

1. **MCP Crate API Mismatch (5 errors)**
   - Root cause: `rmcp` crate API changed, old paths (`rmcp::client::Client`, `rmcp::transport::child_process::ChildProcessTransport`) no longer exist
   - Affected files: `src/mcp/transport/stdio.rs`, `src/mcp/server_manager.rs`

2. **Type Mismatch in Bot Initialization (3 errors)**
   - Root cause: `ai_service` passed as `Future` instead of concrete `AiService` type to `EventHandler::new()`
   - Affected file: `src/bot.rs` lines 157-165

3. **Private Module Access (1 error)**
   - Root cause: `crate::ui::templates` module is private but imported in MCP handlers
   - Affected file: `src/modules/mcp/handlers.rs` line 10

4. **CommandContext Missing Methods (11 errors)**
   - Root cause: MCP handlers use methods (`send_text`, `send_html`, `permission_level`) and `bot` field that don't exist on current `CommandContext` struct
   - Affected file: `src/modules/mcp/handlers.rs`

5. **Other MCP-related errors (18 errors)**
   - Type inference failures, struct field mismatches, lifetime issues, RwLockReadGuard clone error, unstable feature use
   - Affected files: `src/mcp/server_manager.rs`, `src/modules/mcp/handlers.rs`

## Open Questions
✅ All questions answered.

## Git History Findings
- MCP functionality was recently added in commits: `c4421ae` (add MCP support), `709be2f` (integrate MCP to AiService), `8e0a20c` (TOML config support)
- The primary breaking change: `AiService::new()` was changed to async function without updating callers in `bot.rs`
- rmcp version 1.1.0 is being used, API has changed from what the code expects

## Scope Boundaries
- INCLUDE: Analysis of build errors, root cause identification, fix work plan
- EXCLUDE: Refactoring unrelated to build failures, new feature development, complete MCP feature implementation (only fix enough to make build pass)

## Open Questions
- (Pending)

## Scope Boundaries
- INCLUDE: Analysis of build errors, root cause identification, fix work plan
- EXCLUDE: Refactoring unrelated to build failures, new feature development
