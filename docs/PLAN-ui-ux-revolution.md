# PLAN-ui-ux-revolution

## 1. Context Analysis
- **Current State**: 
    - "Void Cyan" Design System is implemented and active.
    - `TableEditor`: 3-column layout exists, but column reordering is visual-only.
    - `NoSqlBrowser`: Master-Detail layout exists, but JSON editing is a basic textarea.
    - General UI: Clean and dark, but lacks transition polish.
- **Goal**: "Enhance this more more" - Move from a static implementation to a dynamic, professional-grade tool.

## 2. Task Breakdown (Phase 2: Refinement & Interactivity)

### 2.1 Table Editor Polish
- [ ] **True Drag-and-Drop**: Implement functional column reordering using HTML5 Drag and Drop API.
- [ ] **Type Selector**: Improve the column type dropdown with better styling/icons.
- [ ] **Keyboard Shortcuts**: Add `Ctrl+S` to apply changes, `Ctrl+N` for new column.

### 2.2 NoSQL Browser Polish
- [ ] **Rich JSON Editor**: Replace raw `textarea` with a syntax-highlighting-friendly component (or enhanced textarea with line numbers).
- [ ] **Metadata Panel**: Create a dedicated section for `_id`, `created_at`, `updated_at` to declutter the main JSON view.
- [ ] **Validation**: Real-time JSON validation with error indicators before submission.

### 2.3 Global UX Enhancements
- [ ] **Transitions**: Add subtle layout transitions (entering pages, switching tabs) using CSS View Transitions or simple animations.
- [ ] **Settings Page**: Implement the Settings UI (theme toggle placeholder, font size preferences).
- [ ] **Loading States**: Replace generic "Loading..." with skeleton screens for Tables/Collections.

## 3. Agent Assignments
- **Frontend Specialist**: All UI components and CSS.
- **Orchestrator**: Coordinating the build and verification.

## 4. Verification Checklist
- [ ] Dragging a column Updates the pending schema state.
- [ ] Invalid JSON in NoSQL editor shows a distinct error state (red border/message).
- [ ] Settings page is accessible and responsive.
- [ ] All animations are smooth (60fps) and do not impede workflow.
