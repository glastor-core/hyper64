# 🛡️ Política de Protección de Rama (`main`) — GLASTOR® Strata

Este documento define la política obligatoria de protección para la rama principal `main` en el repositorio [glastor-core/strata](https://github.com/glastor-core/strata).

---

## 1. Principio Rector

> **Ningún push directo a `main` está permitido.**  
> Todos los cambios sin excepción deben canalizarse a través de un **Pull Request (PR)** verificado que cumpla con los estándares de calidad de GLASTOR® Core.

---

## 2. Reglas Mandatorias de Protección

| Regla | Parámetro | Propósito |
| :--- | :--- | :--- |
| **Require a pull request before merging** | Activado | Impide `git push origin main` directo. |
| **Required approving reviews** | `1` o más | Garantiza revisión por pares (Peer Review). |
| **Require review from Code Owners** | Activado | Exige aprobación de `@glastor-core` o `@glastor-dev` conforme a [`.github/CODEOWNERS`](CODEOWNERS). |
| **Dismiss stale pull request approvals** | Activado | Invalida revisiones previas si se agregan nuevos commits al PR. |
| **Require conversation resolution** | Activado | Todos los hilos y comentarios en el PR deben resolverse antes del merge. |
| **Require status checks to pass** | Activado (Strict) | Exige que la rama esté al día con `main` y que la suite CI pase. |
| **Checks obligatorios requeridos** | 1. `Test Suite & Lints`<br>2. `GLASTOR® Core Audit` | Verifica Rust 2024, `cargo clippy`, `cargo fmt`, `byte-gauge` y `specter`. |
| **Require linear history** | Activado | Fomenta historial limpio (`Squash and merge` o `Rebase and merge`), sin merge commits sucios. |
| **Block force pushes** | Activado (`non_fast_forward`) | Previene reescrituras de historia en `main` (`git push --force`). |
| **Block branch deletions** | Activado (`deletion`) | Previene eliminación accidental de la rama `main`. |

---

## 3. Configuración Automatizada (GitHub CLI)

El archivo de configuración de Ruleset oficial se encuentra versionado en [`.github/rulesets/main-protection.json`](rulesets/main-protection.json).

Para aplicar esta protección inmediatamente usando GitHub CLI autenticado con permisos de admin:

```bash
gh api --method POST /repos/glastor-core/strata/rulesets \
  --input .github/rulesets/main-protection.json
```

O para consultar los rulesets activos:

```bash
gh api /repos/glastor-core/strata/rulesets
```

---

## 4. Configuración Manual en GitHub Web

1. Ir a **Settings** -> **Rules** -> **Rulesets** en `github.com/glastor-core/strata`.
2. Clic en **New branch ruleset**.
3. Asignar el nombre: `strata-main-protection`.
4. Enforcement status: **Active**.
5. Target branches: **Include default branch (`main`)**.
6. Marcar:
   - [x] **Restrict deletions**
   - [x] **Block force pushes**
   - [x] **Require a pull request before merging**
     - Required approvals: `1`
     - Dismiss stale approvals when new commits are pushed: Checked
     - Require review from Code Owners: Checked
     - Require conversation resolution: Checked
   - [x] **Require status checks to pass before merging**
     - Require branches to be up to date before merging: Checked
     - Checks: `Test Suite & Lints`, `GLASTOR® Core Audit`
   - [x] **Require linear history**
7. Clic en **Save changes**.

---

© 2010-2026 Andrés Antonio Cardoso — GLASTOR® Core

