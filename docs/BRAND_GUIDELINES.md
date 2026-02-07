# ESNODE Brand Guidelines

## Logo Assets

### Primary Logo
**File:** `docs/images/esnode-logo-dark.png`

The ESNODE primary logo consists of:
- **Symbol:** Asterisk/star with white and orange accent (✱)
- **Wordmark:** "ESNODE" in clean sans-serif typography
- **Tagline:** "Power-Aware AI Infrastructure"
- **Background:** Dark navy (#1a2332)

**Usage:**
- README headers
- Documentation
- Marketing materials
- Presentations

### Icon/Avatar Logo
**File:** `docs/images/esnode-icon.png`

Compact square icon featuring:
- **Symbol:** Asterisk/star (white with orange accent)
- **Background:** Black (#000000)
- **Format:** Square, suitable for app icons and avatars

**Usage:**
- GitHub/GitLab avatars
- Application icons
- Social media profiles
- Favicons

---

## Color Palette

### Primary Colors

#### Dark Navy (Brand Background)
- **RGB:** `26, 35, 50`
- **Hex:** `#1a2332`
- **Usage:** Main backgrounds, headers, cards

#### Sidebar Navy (Darker Variant)
- **RGB:** `20, 27, 40`
- **Hex:** `#141b28`
- **Usage:** Sidebar backgrounds, secondary panels

#### Orange/Amber Accent
- **RGB:** `255, 193, 7`
- **Hex:** `#ffc107`
- **Usage:** Logo accent, warning states, highlights

### Semantic Colors

#### Success/Healthy Green
- **RGB:** `76, 175, 80`
- **Hex:** `#4caf50`
- **Usage:** Success indicators, healthy status, OK states

#### Information/Active Blue
- **RGB:** `37, 99, 235`
- **Hex:** `#2563eb`
- **Usage:** Active selections, links, primary actions

#### Light Blue (Labels)
- **RGB:** `100, 181, 246`
- **Hex:** `#64b5f6`
- **Usage:** Labels, secondary text, data headers

#### Warning/Alert Red
- **RGB:** `244, 67, 54 `
- **Hex:** `#f44336`
- **Usage:** Errors, critical warnings, thermal alerts

#### Light Gray (Text)
- **RGB:** `156, 163, 175`
- **Hex:** `#9ca3af`
- **Usage:** Secondary text, disabled states

### Text Colors

#### Primary Text
- **Color:** White (`#ffffff`)
- **Usage:** Headers, primary content

#### Secondary Text
- **Color:** Light Gray (`#9ca3af`)
- **Usage:** Descriptions, metadata, timestamps

---

## Typography

### TUI Display
- **Font:** Monospace terminal fonts
- **Header:** Bold weight
- **Body:** Regular weight
- **Logo:** Bold capitals for "ESNODE"

### Documentation
- **Headings:** Sans-serif (GitHub default)
- **Body:** Readable sans-serif
- **Code:** Monospace

---

## TUI Design System

### Layout Structure

```
┌─────────────────────────────────────────────────────┐
│ ✱ ESNODE  Power-Aware AI Infrastructure   ● ONLINE │ ← Header
├──────────┬──────────────────────────────────────────┤
│          │                                          │
│ Nav      │ Content Area                             │
│ Sidebar  │ (Gauges, Tables, Charts)                 │
│          │                                          │
│          │                                          │
├──────────┴──────────────────────────────────────────┤
│ F5: Refresh | Arrow Keys: Navigate | Q/F3: Quit    │ ← Footer
└─────────────────────────────────────────────────────┘
```

### Component Styles

#### Headers
- **Background:** Dark Navy (#1a2332)
- **Border:** Bottom border only
- **Height:** 3 lines
- **Content:** Logo, tagline, status indicator

#### Sidebar Navigation
- **Background:** Sidebar Navy (#141b28)
- **Text:** Light Gray (#9ca3af)
- **Active Item:** Blue background (#2563eb), white text, bold
- **Selection Indicator:** Arrow `▶` symbol
- **Width:** 25% of terminal width

#### Content Area
- **Background:** Terminal default (black)
- **Borders:** Box-drawing characters
- **Padding:** 2 characters horizontal, 1 vertical

#### Footer
- **Background:** Dark gray
- **Text:** White
- **Height:** 1 line
- **Content:** Keyboard shortcuts

### Widget Styles

#### Tables
- **Headers:** Light blue (#64b5f6)
- **Borders:** All sides
- **Alternating Rows:** Optional subtle background

#### Gauges
- **Bar Color:** Green (#4caf50) for normal, Red (#f44336) for critical
- **Background:** Dark
- **Border:** All sides
- **Label:** Percentage or value display

#### Lists
- **Bullet:** None or simple dash
- **Spacing:** Single line between items
- **Icons:** Color-coded status indicators (●)

#### Status Indicators
- **Online:** Green `● ONLINE`
- **Connecting:** Amber `● CONNECTING...`
- **Offline:** Red `● OFFLINE`
- **Warning:** Amber `⚠ WARNING`

---

## Icon System

### Status Icons
- `✱` - ESNODE logo symbol
- `●` - Status indicator (colored)
- `▶` - Active selection arrow
- `⚠` - Warning/alert
- `✓` - Success/checkmark
- `✗` - Error/failure

### Navigation Icons
- `↑` / `↓` - Scroll/navigate up/down
- `←` / `→` - Navigate left/right
- `⏎` - Enter/confirm

---

## Usage Examples

### TUI Header Code (Rust)
```rust
let header_text = Line::from(vec![
    Span::styled("✱ ", Style::default().fg(Color::Rgb(255, 193, 7))),
    Span::styled("ESNODE", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    Span::raw("  "),
    Span::styled("Power-Aware AI Infrastructure", Style::default().fg(Color::Rgb(100, 181, 246))),
    Span::raw("                    "),
    Span::styled("● ONLINE", Style::default().fg(Color::Rgb(76, 175, 80))),
]);
```

### README Badge Colors
- **License Badge:** Blue (#0969da)
- **Build Status:** Green (#00c851) for passing, Red (#f44336) for failing
- **Version Badge:** Orange (#ffc107)

---

## Brand Voice

### Tone
- **Professional:** Enterprise-grade quality
- **Technical:** Precise, accurate, data-driven
- **Confident:** Leading-edge technology
- **Accessible:** Clear, understandable documentation

### Messaging
- **Primary Message:** "Power-Aware AI Infrastructure"
- **Value Proposition:** Real-time observability for AI workloads with energy efficiency
- **Differentiator:** Autonomous power-aware orchestration

---

## File Organization

```
docs/
├── images/
│   ├── esnode-logo-dark.png      # Primary horizontal logo
│   ├── esnode-icon.png            # Square icon version
│   ├── tui-overview.png           # TUI screenshot - Overview
│   ├── tui-gpu.png                # TUI screenshot - GPU view
│   └── tui-orchestrator.png       # TUI screenshot - Orchestrator
└── branding/
    ├── BRAND_GUIDELINES.md        # This file
    └── color-palette.md           # Detailed color specifications
```

---

## Consistency Checklist

When creating new materials, ensure:

- [x] ESNODE logo uses asterisk symbol (✱)
- [x] Tagline is "Power-Aware AI Infrastructure" (not alternatives)
- [x] Dark navy background matches brand (#1a2332)
- [x] Orange accent (#ffc107) is used sparingly
- [x] Status indicators use semantic colors (green/amber/red)
- [x] Typography is clean and professional
- [x] TUI maintains 3-line header + footer + sidebar structure
- [x] All interactive elements have keyboard shortcuts documented

---

## Version History

**v1.0** - 2026-02-07
- Initial brand guidelines
- TUI color scheme modernization
- Logo asset integration
- Comprehensive color palette definition

---

**Maintained by:** ESNODE Team  
**Last Updated:** 2026-02-07  
**Contact:** [Repository Issues](https://github.com/ESNODE/ESNODE-Core/issues)
