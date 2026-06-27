# Стандарты качества кода

Проект: `kaerna-group/launcher` — Tauri + React + TypeScript (frontend), Rust (backend).

---

## Быстрый старт

```bash
task check   # полная проверка (rustfmt + clippy + tsc + lint + prettier)
task fix     # авто-исправить что можно (fmt + lint:fix + prettier)
```

Отдельные задачи:

```bash
task fmt:check     # только rustfmt --check
task clippy        # только clippy
task ts:check      # только TypeScript type-check
task lint          # только ESLint
task prettier:check  # только Prettier
```

Или напрямую через pnpm (из папки `launcher/`):

```bash
pnpm type-check      # Проверка типов TypeScript
pnpm lint            # ESLint
pnpm lint:fix        # ESLint с автоисправлением
pnpm format:check    # Prettier (только проверка)
pnpm format          # Prettier (форматирование)
pnpm check           # type-check + lint + format:check
```

---

## Rust

**Конфиги:** [`src-tauri/rustfmt.toml`](../launcher/src-tauri/rustfmt.toml), [`src-tauri/Cargo.toml`](../launcher/src-tauri/Cargo.toml) (`[lints]`)

### Форматирование — rustfmt

| Правило | Значение | Причина |
|---|---|---|
| `max_width` | 100 | Баланс читаемости и компактности |
| `tab_spaces` | 4 | Стандарт Rust |
| `newline_style` | `Unix` | LF везде, включая Windows |
| `reorder_imports` | `true` | Алфавитная сортировка use-блоков |
| `reorder_modules` | `true` | Алфавитная сортировка mod-деклараций |
| `match_block_trailing_comma` | `true` | Упрощает diff при добавлении веток |

> Опции `imports_granularity`, `group_imports`, `trailing_comma`, `wrap_comments` и др. доступны только на nightly — не используются.

Запуск:
```bash
cd launcher/src-tauri
cargo fmt --all               # форматировать
cargo fmt --all -- --check    # только проверить
```

### Линтинг — Clippy

Включены группы **pedantic** и **nursery** как `warn`. В CI флаг `-D warnings` превращает все предупреждения в ошибки.

| Правило | Уровень |
|---|---|
| `unsafe_code` | `forbid` — абсолютный запрет |
| `unused_imports` | `warn` |
| `dead_code` | `warn` |
| `clippy::pedantic` | `warn` (priority -1) | Весь pedantic-набор |
| `clippy::nursery` | `warn` (priority -1) | Экспериментальные, но полезные |
| `module_name_repetitions` | `allow` | Нормальная практика в Rust |
| `must_use_candidate` | `allow` | Слишком шумно |
| `missing_errors_doc` | `allow` | doc-errors не обязательны в приватном коде |
| `missing_panics_doc` | `allow` | doc-panics не обязательны в приватном коде |
| `too_many_arguments` | `allow` | Builder-паттерн оправдывает много аргументов |

Запуск:
```bash
cd launcher/src-tauri
cargo clippy --all-targets --all-features -- -D warnings
```

### Правила написания Rust-кода

- **Нет `unwrap()` / `expect()` в библиотечном коде.** Используй `?` и `thiserror`.
- **Нет `clone()` без причины.** Передавай ссылки.
- **`unsafe` — запрещён** (`unsafe_code = "forbid"`).
- **Именование:** `snake_case` для переменных/функций, `PascalCase` для типов, `SCREAMING_SNAKE_CASE` для констант.
- **Модули:** один файл — одна ответственность. Публичный API — только то, что нужно снаружи.
- **Логирование:** через `tracing` (`info!`, `warn!`, `error!`). Не используй `println!` в production-коде.
- **Комментарии:** только когда `why` не очевиден из кода.

---

## TypeScript / React

**Конфиги:** [`launcher/eslint.config.js`](../launcher/eslint.config.js), [`launcher/.prettierrc`](../launcher/.prettierrc)

### Форматирование — Prettier

| Параметр | Значение |
|---|---|
| `semi` | `false` — без точки с запятой |
| `singleQuote` | `true` |
| `trailingComma` | `all` |
| `printWidth` | 100 |
| `tabWidth` | 2 |
| `endOfLine` | `lf` |

### Линтинг — ESLint

Используется **ESLint 9 flat config** + `typescript-eslint/strict` + react-hooks.

| Правило | Уровень | Причина |
|---|---|---|
| `@typescript-eslint/no-explicit-any` | `error` | `any` убивает смысл TypeScript |
| `@typescript-eslint/consistent-type-imports` | `error` | Разделяй value и type imports |
| `@typescript-eslint/no-non-null-assertion` | `warn` | Используй optional chaining |
| `no-console` | `warn` (allow warn/error) | Нет `console.log` в production |
| `prefer-const` | `error` | Неизменяемые привязки по умолчанию |
| `no-duplicate-imports` | `error` | Один import-блок на модуль |
| `react-hooks/rules-of-hooks` | `error` | Хуки только на верхнем уровне |
| `react-hooks/exhaustive-deps` | `warn` | Полные зависимости в useEffect |

### Правила написания TS/React-кода

- **Нет `any`.** Если тип неизвестен — `unknown`, затем сужай через guard.
- **Explicit return types** у публичных функций и хуков.
- **`type` vs `interface`:** `interface` для объектов, расширяемых снаружи; `type` для union/intersection.
- **Импорты:** сначала внешние пакеты, затем внутренние. Используй `type` import для типов.
- **Именование:** компоненты — `PascalCase`; хуки — `use` + `PascalCase`; утилиты — `camelCase`.
- **Нет inline стилей.** Используй CSS-модули или файлы в `shared/styles/`.
- **Структура компонента:** props type → component → export. Без default export в файлах с несколькими экспортами.
- **Нет `console.log`.** Только `console.warn` и `console.error` там, где это оправдано.

---

## Архитектура (Feature-Sliced Design)

```
src/
  app/       — провайдеры, рутовые стили
  pages/     — страницы (маршруты)
  widgets/   — самодостаточные UI-блоки
  features/  — изолированные фичи с состоянием
  entities/  — доменные модели (без UI)
  shared/    — утилиты, ui-kit, стили без бизнес-логики
```

**Правило импортов:** нижние слои не знают о верхних.
`shared` → `entities` → `features` → `widgets` → `pages` → `app`

---

## CI — GitHub Actions

Конфиг: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

Запускается на каждый push/PR в `master`. Два параллельных job-а:

| Job | Шаги | Кэш |
|---|---|---|
| **Rust** | fmt → clippy `-D warnings` → test | `Swatinem/rust-cache` по `Cargo.lock` |
| **Frontend** | type-check → lint → prettier → build | pnpm store по `pnpm-lock.yaml` |

Job **`ci`** — единственная точка для branch protection rule в GitHub Settings → Branches.

**Особенности:**
- `concurrency` с `cancel-in-progress: true` — устаревшие запуски отменяются при новом push
- `CARGO_INCREMENTAL: 0` — инкрементальная компиляция замедляет CI, отключена
- `pnpm install --frozen-lockfile` — lockfile не может быть изменён в CI
- Rust job устанавливает `libwebkit2gtk-4.1-dev` и другие системные зависимости Tauri

**Branch protection (настроить на GitHub):**  
Settings → Branches → Add rule → Require status checks → добавить `CI`.
