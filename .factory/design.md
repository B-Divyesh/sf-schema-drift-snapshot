# Schema Drift Snapshot — visual thesis

## Direction: paper-cut incident diorama

The product turns a flat, intimidating catalog dump into layers a reviewer can
lift apart. The site therefore behaves like a paper-cut diorama on a database
engineer's workbench: the expected schema is warm vellum at the back, production
is a cool blue sheet in front, and drift appears as a coral cut edge between
them. This depth is explanatory rather than decorative; the same layered shape
language marks before, after, risk, and ownership in the report demo.

The treatment is deliberately single-mode, like a printed incident packet under
daylight. A dark theme would weaken the physical-paper metaphor. The background
is explicitly painted in every route.

## Tokens

| Role | Token | Value | Source in the product world |
| --- | --- | --- | --- |
| canvas | `--paper` | `#F5F0E5` | warm incident-review paper |
| raised sheet | `--sheet` | `#FFFDF7` | unmarked catalog page |
| ink | `--ink` | `#182723` | near-black database-console ink |
| muted ink | `--muted` | `#50605B` | pencil annotation |
| expected | `--blue` | `#1F6675` | blueprint layer |
| expected dark | `--blue-deep` | `#164B57` | accessible controls |
| drift | `--coral` | `#C34F3D` | exposed cut edge / danger |
| drift dark | `--coral-deep` | `#8D3025` | accessible danger text |
| caution | `--ochre` | `#8B6414` | review flag |
| safe | `--fern` | `#2F6B4F` | verified, additive change |
| shadow | `--shadow` | `#19383024` | physical layer separation |

All body copy is at least 4.5:1 against its surface. State always combines
color with a label, icon, or sentence.

## Type and spacing

No font files or remote requests are needed. Headings use the editorial,
paper-like local stack `Iowan Old Style, Palatino Linotype, Book Antiqua,
Georgia, serif`; utility copy and code use `ui-monospace, SFMono-Regular,
Consolas, Liberation Mono, monospace`. The contrast evokes a human review note
wrapped around machine evidence while keeping first-load font bytes at zero.

The scale is 16, 18, 23, 31, 46, and 68px with 1.5 line-height for prose.
Spacing follows an 8px rhythm with 4px for inline optical corrections. Prose
measures 68 characters. Touch targets are at least 44px.

## Composition and components

- Sheets use one clipped corner, a 1px ink edge, and a short offset shadow.
- The hero diorama shows two catalog layers pulled apart at one drift seam.
- Risk chips look like small review tabs, not generic rounded pills.
- Tables flatten on phones into labelled change blocks; no horizontal task is
  required at 390px.
- The only primary accent action is blueprint blue. Coral is reserved for
  detected risk and never used as a marketing button.

## Interaction grammar and motion

Controls depress by 2px as if pressing paper against a desk. The demo changes
layers with a 220ms opacity/translate transition; disclosure arrows rotate
from their hinge in 180ms. Nothing loops. Under `prefers-reduced-motion`, all
translations and rotations are removed and state changes are instant opacity
changes. Depth survives through borders, overlap, and static shadows.

## Original asset plan and provenance

`site/public/assets/schema-diorama.webp` is an original raster hero generated
for this product with the factory image deployment on 2026-08-27, then locally
resized and encoded to WebP. Prompt: “Editorial paper-cut diorama of two database
schema blueprints on a database engineer's desk; layered warm vellum and deep
teal paper, one precise coral seam showing drift, tiny cut-paper columns and
relationship lines, soft directional shadows, tactile fibers, no people, no
logos, no text, no gradients, wide landscape, sophisticated technical
illustration, clean negative space.” Model/deployment metadata is stored beside
the generated source during production; the optimized WebP is the shipped
original asset. License: project-owned generated work. All icons are original
inline CSS/SVG geometric marks derived from catalog columns; no stock assets.
