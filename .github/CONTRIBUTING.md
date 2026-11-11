# 🌌 Guía de Contribución y Estándares de Ingeniería — GLASTOR® Strata

Gracias por sumarte a la misión de construir y evolucionar **GLASTOR® Strata** dentro del ecosistema de `GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**).

Strata es la infraestructura definitiva de interfaces de usuario y sistema de diseño headless para el ecosistema **Rust WebAssembly** (Leptos, Dioxus, Yew y WASM nativo), eliminando por completo Node.js y npm.

---

## 🛡️ Política de Protección de Rama (`main`)

Para garantizar la estabilidad y la calidad del código en producción:
- **La rama `main` está protegida**: No se permiten envíos directos (`git push origin main`) ni modificaciones forzadas (`--force`).
- **Todo cambio debe ingresar mediante Pull Request (PR)**:
  - Debe crearse una rama temática (`feat/nombre`, `fix/nombre`).
  - La suite automatizada de GitHub Actions (formato, linter Clippy, tests unitarios y auditoría Specter) debe finalizar en estado verde (`[PASS]`).
  - Se requiere al menos una revisión aprobada por los mantenedores ([CODEOWNERS](./CODEOWNERS)).

---

## 💻 Setup del Entorno de Desarrollo

### Requisitos Previos

- **Rust**: Versión estable 1.85+ (Edición 2024 recomendada).
- **Git**: 2.30+.
- **Zero Node.js**: No se requiere instalar Node.js, npm, yarn ni pnpm.

### 1. Clonación del Repositorio

```bash
git clone https://github.com/glastor-core/strata.git
cd strata
```

### 2. Comandos de Trabajo

```bash
# Compilar todos los crates de la workspace
cargo build --workspace --all-targets

# Ejecutar las 30 pruebas unitarias e integrales
cargo test --workspace --all-features

# Verificar cumplimiento de formato oficial
cargo fmt --all --check

# Ejecutar auditoría estricta de Clippy (pedantic & nursery)
cargo clippy --workspace --all-targets --all-features

# Validar auditoría de huella de memoria (byte-gauge)
cargo test -p glastor-aria-core --test memory_footprint

# Auditar contratos AST de componentes (specter)
cargo run --bin cargo-ui -- audit

# Sincronizar tokens de diseño W3C Figma / Penpot
cargo run --bin cargo-ui -- sync-tokens --input tokens.json
```

---

## 📐 Estándares de Código y Arquitectura

1. **`#![forbid(unsafe_code)]`**:
   No se acepta código `unsafe` en ninguna parte de la base de código.
2. **Determinismo FSM**:
   Toda interacción en `glastor-aria-core` debe modelarse como una máquina de estados pura (`StateMachine`) que implemente `next_transition`.
3. **Huella de Memoria Minimizada**:
   Los estados enum interactivos deben ocupar $\le 1$ byte para optimizar el paso de mensajes en Wasm.
4. **Sanitización de CSS en `glastor-css-binder`**:
   Los valores arbitrarios deben ser sanitizados contra secuencias de inyección (`url(`, `expression(`, `@import`, `;`, `{`, `}`).
5. **Mensajes de Commit**:
   Sigue estrictamente la especificación [Conventional Commits](./COMMIT_CONVENTION.md).

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
