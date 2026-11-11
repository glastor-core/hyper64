# 🏛️ Convención de Mensajes de Commit (Conventional Commits) — GLASTOR® Strata

Para escalar el desarrollo y garantizar una **Software Supply Chain** transparente, este repositorio implementa una convención estricta basada en el estándar [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

Esta práctica impulsa la automatización de versiones con SemVer, la generación de changelogs y la auditoría de contratos con `specter`.

---

## ⚙️ Estructura del Mensaje

```text
<tipo>(<scope>): <asunto en minúsculas y modo imperativo>

[cuerpo descriptivo opcional]

[pie de página / referencias opcional]
```

### 1. Tipos de Commit Permitidos

| Tipo | Propósito | Impacto en Release |
| :--- | :--- | :---: |
| `feat` | Nueva característica o primitiva FSM | Minor (`v0.X.0`) |
| `fix` | Corrección de bug o comportamiento inesperado | Patch (`v0.0.X`) |
| `perf` | Optimización de memoria, SIMD o tiempo de compilación | Patch (`v0.0.X`) |
| `refactor` | Reestructuración de código sin alterar comportamiento | Ninguno |
| `docs` | Actualización de documentación, `llms.txt` o specs | Ninguno |
| `test` | Nuevas pruebas unitarias, de integración o footprints | Ninguno |
| `ci` | Ajustes en GitHub Actions, workflows o automatizaciones | Ninguno |
| `chore` | Tareas de mantenimiento, dependencias o tooling | Ninguno |

### 2. Scopes Oficiales de Strata

- `aria`: Primitivas y máquinas de estado en `glastor-aria-core` (`dialog`, `accordion`, `tabs`, `toast`, `grid`).
- `css`: Motor de escaneo y compilación de CSS en `glastor-css-binder`.
- `motion`: Físicas de resortes y generación de keyframes GPU (`glastor-motion`).
- `cli`: Herramienta de línea de comandos `cargo ui` / `cargo strata`.
- `tokens`: Sincronizador de Design Tokens W3C (`sync-tokens`).
- `assets`: Iconografía vectorial SVG, favicons y manifiestos PWA.
- `landing`: Página web interactiva de demostración (`index.html`).

---

## 💡 Ejemplos Válidos

```text
feat(grid): implement 2d roving arrow navigation and virtual slicing
fix(css): prevent arbitrary border colors from overriding border-b
perf(fsm): optimize dialog transition loop to sub-microsecond latency
docs(tokens): document W3C DTCG json schema synchronization
ci(main): enforce branch protection and cargo audit checks
```

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
