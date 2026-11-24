# Builder plan — macos-support-and-eye-comfort

BA audit: PASS

## Phase 1 — Architecture & Plan Lock

Agents: planner, ags-advisor, advisor
Skills: ags-plan, ags-advise, ags-fable-thinking, ags-vibe, ags-context-engineering

- [ ] Lock scope from docs/macos-support-and-eye-comfort/srs/spec.md: 4 BR, 4 error codes
- [ ] Confirm 1 screen(s), 5 local CLI/MCP interface(s), 1 user story file(s) before coding

## Phase 2 — Frontend UX/UI

Agents: ui-ux-designer
Skills: ags-frontend-design, ags-ui-ux-pro-max, ags-ui-styling, ags-frontend-development, ags-page-builder, ags-react-best-practices, ags-tanstack, ags-threejs, ags-shader, ags-remotion, ags-hyperframes, ags-preview

- [ ] Build screen main (docs/macos-support-and-eye-comfort/ascii-screen/main.md) with Loading/Empty/Error/Populated states; data-testid: bearrrr-theme-card, macos-install-action, macos-launch-action, macos-status-panel, macos-uninstall-action

## Phase 3 — Backend API & DB

Agents: fullstack-developer
Skills: ags-backend-development, ags-databases, ags-better-auth, ags-payment-integration, ags-mcp-builder, ags-devops, ags-deploy, ags-security

- [ ] Implement local CLI/MCP interface: Surface: macos-install-action; CLI: agy-theme install; MCP: none; Input: --no-startup, --watch-themes, --root; Output: stdout install report (PID, RAM MB, LaunchAgent & App path); Persistence: theme_config.json, ~/Library/LaunchAgents/ (plist), ~/Applications/ (launcher bundle), dist payload; Errors: E-macos-support-and-eye-comfort-002, E-macos-support-and-eye-comfort-003, E-macos-support-and-eye-comfort-004
- [ ] Implement local CLI/MCP interface: Surface: macos-launch-action; CLI: agy-theme launch; MCP: none; Input: --restart, --root; Output: stdout launch report + spawned Antigravity process with --remote-debugging-port=0; Persistence: theme_config.json (reads active config & starts daemon); Errors: E-macos-support-and-eye-comfort-001, E-macos-support-and-eye-comfort-002
- [ ] Implement local CLI/MCP interface: Surface: macos-status-panel; CLI: agy-theme status; MCP: none; Input: --root; Output: stdout status lines (Root, Payload KB, Daemon PID/RSS, CDP port); Persistence: none (read-only); Errors: E-macos-support-and-eye-comfort-003
- [ ] Implement local CLI/MCP interface: Surface: macos-uninstall-action; CLI: agy-theme uninstall; MCP: none; Input: --root; Output: stdout uninstall report (removed LaunchAgent, launcher app, cleared CDP pages); Persistence: theme_config.json (preserved), removes ~/Library/LaunchAgents/ plist & ~/Applications/ launcher bundle; Errors: E-macos-support-and-eye-comfort-002
- [ ] Implement local CLI/MCP interface: Surface: bearrrr-theme-card; CLI: agy-theme build; MCP: none; Input: themes/themes.json, engine/core.js, --max-dim; Output: dist payload + build elapsed_ms & theme list; Persistence: themes/themes.json, theme_config.json; Errors: E-macos-support-and-eye-comfort-004
- [ ] Enforce business rules: BR-macos-support-and-eye-comfort-001, BR-macos-support-and-eye-comfort-002, BR-macos-support-and-eye-comfort-003, BR-macos-support-and-eye-comfort-004
- [ ] Return error codes: E-macos-support-and-eye-comfort-001, E-macos-support-and-eye-comfort-002, E-macos-support-and-eye-comfort-003, E-macos-support-and-eye-comfort-004

## Phase 4 — BDD Verification & Review

Agents: tester, code-reviewer, code-simplifier
Skills: ags-test, ags-web-testing, ags-agent-browser, ags-code-review, ags-review-pr, ags-skill-creator, ags-ship

- [ ] Automate docs/macos-support-and-eye-comfort/userstories/us-001.md: 5 Given/When/Then scenario(s) via E2E
- [ ] Review diff against srs/spec.md, api-map.md and the prototype before merge

## Generated from

- docs/macos-support-and-eye-comfort/api/api-map.md
- docs/macos-support-and-eye-comfort/ascii-screen/_index.md
- docs/macos-support-and-eye-comfort/ascii-screen/main.md
- docs/macos-support-and-eye-comfort/html-design/prototype.html
- docs/macos-support-and-eye-comfort/srs/spec.md
- docs/macos-support-and-eye-comfort/srs/user-flow.md
- docs/macos-support-and-eye-comfort/userstories/us-001.md
