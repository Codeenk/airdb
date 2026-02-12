# AirDB Next Steps Plan

## Phase 1: Feature Verification Testing (Immediate)

### 1.1 Migration System Testing
- [ ] Test new table creation with migration generation
  - Create table with columns
  - Verify `isNew: true` parameter in migration
  - Confirm migration file created in correct location
- [ ] Test ALTER TABLE migration generation
  - Add new column to existing table
  - Modify column type
  - Verify `isNew: false` parameter in migration
- [ ] Test migration history viewer
  - Open Settings → Migrations tab
  - Verify all past migrations display
  - Check migration rollback functionality

### 1.2 UI Component Testing
- [ ] Column type dropdown visibility
  - Open Table Editor
  - Click column type dropdown
  - Verify all SQL types visible (TEXT, INTEGER, REAL, BLOB)
  - Confirm styling matches design
- [ ] ConfirmDialog component
  - Trigger delete operations
  - Verify danger/warning/info variants
  - Test ESC key dismissal
  - Test overlay click dismissal

### 1.3 Authentication Testing
- [ ] GitHub auth persistence
  - Log in with GitHub token
  - Close application completely
  - Restart application
  - Verify auto-login within 30-day window
  - Test token expiry handling
- [ ] Test auth-storage utility
  - saveToken() with expiry
  - loadToken() validity check
  - clearToken() cleanup

### 1.4 Hybrid SQL Editor Testing
- [ ] Visual mode functionality
  - Create table visual builder
  - Add/remove columns
  - Set constraints
- [ ] Raw SQL mode functionality
  - Toggle to Raw mode (button click)
  - Type SELECT query
  - Execute with Cmd+S (Mac) / Ctrl+S (Linux)
  - View results in table format
- [ ] Mode switching
  - Visual → Raw transition
  - Raw → Visual transition
  - Verify state preservation

### 1.5 Keyboard Shortcuts Testing
- [ ] Cmd+S / Ctrl+S - Save/Execute
  - In Visual mode: Save table schema
  - In Raw mode: Execute SQL query
- [ ] Cmd+N / Ctrl+N - New table
  - Opens new table creation dialog
- [ ] ESC - Cancel/Close
  - Closes dialogs
  - Exits edit mode

### 1.6 NoSQL Initialization Testing
- [ ] Uninitialized project handling
  - Create new project
  - Click NoSQL tab before initialization
  - Verify helpful error message appears
  - Confirm `isInitialized` check working
- [ ] Post-initialization
  - Initialize NoSQL collections
  - Verify collections browser displays
  - Test collection CRUD operations

### 1.7 Backend Command Testing
- [ ] execute_raw_sql command
  - CREATE TABLE statement
  - SELECT query with multiple rows
  - INSERT with values
  - UPDATE existing records
  - Verify JSON result format
- [ ] get_project_type command
  - SQL-only project → returns "sql"
  - NoSQL-only project → returns "nosql"
  - Hybrid project → returns "hybrid"
- [ ] set_project_type command
  - Change from sql to hybrid
  - Verify config file updated
  - Confirm UI reflects change
- [ ] Lock system commands
  - is_update_blocked during migration
  - get_active_locks returns current locks
  - check_lock verifies lock status

### 1.8 Linux WebKit Testing
- [ ] Development environment
  - Run with `npm run tauri:dev`
  - Verify env-setup.sh executes
  - Confirm snap variables unset
  - Check system WebKit libraries loaded
- [ ] Build environment
  - Run `npm run tauri:build`
  - Verify production bundling
  - Test on non-snap system

## Phase 2: Code Quality Improvements (Low Priority)

### 2.1 Fix Rust Warnings
- [ ] locks.rs line 4
  - Remove unused `use std::path::PathBuf;`
- [ ] locks.rs line 47
  - Prefix with underscore: `_lock_type` or remove
- [ ] locks.rs line 55
  - Prefix with underscore: `_locks` or remove
- [ ] Re-run `cargo check` to verify 0 warnings

### 2.2 TypeScript Refinements
- [ ] Add strict null checks if not enabled
- [ ] Review any `any` types for stricter typing
- [ ] Consider adding ESLint rules

## Phase 3: Documentation Updates

### 3.1 Update IMPLEMENTATION_COMPLETE.md
- [ ] Mark ConfirmDialog as ✅ COMPLETE (recreated)
- [ ] Add backend command documentation
  - execute_raw_sql parameters and return format
  - get/set_project_type usage examples
  - Lock system command examples
- [ ] Add verification status section
- [ ] Include testing results

### 3.2 User Documentation
- [ ] Update docs/quickstart.md
  - Add hybrid SQL editor section
  - Document keyboard shortcuts
  - Show Raw SQL mode examples
- [ ] Update docs/sql-guide.md
  - Raw SQL query examples
  - Migration generation process
  - Best practices for schema changes
- [ ] Update docs/nosql-guide.md
  - Initialization steps
  - Error handling guide

### 3.3 Developer Documentation
- [ ] Create CONTRIBUTING.md
  - Development setup (WebKit fix)
  - Build process
  - Testing guidelines
- [ ] Create ARCHITECTURE.md
  - Frontend architecture (React + Tauri)
  - Backend architecture (Rust commands)
  - Data flow diagrams
  - Lock system design

## Phase 4: CLI Implementation

### 4.1 Core CLI Commands
- [ ] `airdb init`
  - Initialize new project
  - Create database.db and config
  - Set project type (--sql, --nosql, --hybrid)
- [ ] `airdb keys`
  - Generate and display API keys
  - Manage key rotation
  - List active keys
- [ ] `airdb migrate`
  - Apply pending migrations
  - Rollback migrations
  - Migration status
- [ ] `airdb sync`
  - Push to GitHub
  - Pull from GitHub
  - Conflict resolution

### 4.2 CLI Infrastructure
- [ ] Argument parsing with clap
- [ ] Configuration file handling
- [ ] Error reporting and logging
- [ ] Integration with desktop app

## Phase 5: Production Deployment Preparation

### 5.1 Release Engineering
- [ ] Version management
  - Semantic versioning strategy
  - Changelog automation
  - Release notes generation
- [ ] Build pipeline
  - Multi-platform builds (Linux, macOS, Windows)
  - Code signing
  - Auto-update mechanism testing

### 5.2 Distribution
- [ ] Linux packaging
  - .deb package
  - .rpm package
  - AppImage
  - Snap (with WebKit fix documented)
- [ ] Update server setup
  - Manifest generation
  - Checksum verification
  - Rollback capability

### 5.3 Security Hardening
- [ ] Authentication security audit
  - Token storage encryption
  - GitHub OAuth flow review
  - Session management validation
- [ ] SQL injection prevention
  - Verify parameterized queries
  - Test execute_raw_sql security
- [ ] Lock system security
  - Race condition testing
  - Concurrent operation handling

## Phase 6: Advanced Features (Future)

### 6.1 Enhanced UI Components
- [ ] Column type icons in TableEditor
- [ ] Visual foreign key selector
  - Dropdown of available tables
  - Visual relationship diagram
- [ ] Query result export
  - CSV export
  - JSON export
  - SQL dump

### 6.2 Collaboration Features
- [ ] Real-time conflict detection
- [ ] Merge conflict resolution UI
- [ ] Team activity feed
- [ ] Comment system for schema changes

### 6.3 Performance Optimization
- [ ] Large dataset handling
  - Pagination for query results
  - Virtual scrolling
  - Lazy loading
- [ ] Query optimization suggestions
- [ ] Index recommendation engine

## Success Criteria

### Phase 1 Complete When:
- ✅ All 8 feature verification tests pass
- ✅ No TypeScript compilation errors
- ✅ No Rust compilation errors or warnings
- ✅ All keyboard shortcuts functional
- ✅ Authentication persistence confirmed

### Phase 2 Complete When:
- ✅ `cargo check` shows 0 warnings
- ✅ Code follows style guidelines
- ✅ No unused imports or variables

### Phase 3 Complete When:
- ✅ All documentation updated
- ✅ User guides complete with examples
- ✅ Developer onboarding streamlined

### Phase 4 Complete When:
- ✅ CLI commands functional
- ✅ Desktop + CLI integration seamless
- ✅ Help documentation complete

### Phase 5 Complete When:
- ✅ Multi-platform builds successful
- ✅ Auto-update system tested
- ✅ Security audit complete

## Immediate Next Action

**START HERE**: Run Phase 1.1 - Migration System Testing
1. Open AirDB desktop app
2. Create new test project
3. Create table "users" with columns (id, name, email)
4. Check migrations folder for generated file
5. Verify `isNew: true` in migration
6. Add column "created_at" to "users"
7. Verify `isNew: false` in new migration
8. Report results

**Estimated Time**: 2-3 hours for complete Phase 1 verification
