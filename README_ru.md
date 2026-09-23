# AAGL (Агентный Язык Архитектурных Графов)

**AAGL** — это открытый, ориентированный на LLM и агентов пространственный стандарт данных и специализированная СУБД, разработанная для замены устаревших форматов САПР (DXF/IFC). Она предоставляет детерминированную, высокопроизводительную систему на основе графов, специально созданную для того, чтобы автономные ИИ-агенты могли выполнять надёжные параллельные пространственные и структурные правки.

---

## ✅ Текущий статус реализации (v0.1.0 — Core Engine MVP)

- [x] **Слой контейнера (§ Container Layer)**: Формат `.aagl` (ZIP) и плоский Folder-режим (git-friendly dual-backend)
- [x] **Слой графа (§ Graph Layer)**: Universal Graph — 4-полевая нода (`parent_id`, `coords: Vec<i64>`, `relations`, `metadata`), чанки, генезис-куб с дефолтными границами ±500_000
- [x] **Clean Architecture (4 слоя)**:
    - `common/` — константы (`SPEC_VERSION`, `CREATE_AS_FOLDER`, `DEFAULT_GENESIS_SPACE_SIZE` …) + единый enum ошибок `AaglError` + `Result<T>`
    - `domain/` — чистые структуры `Manifest`, `Chunk`, `Node`, `Relation`, `Theme`, `PatchContext` (без логики, без IO)
    - `application/` — use cases: `create_empty_project`, 3 рабочих мутации `add_object` / `delete_object` / `update_object` (single-chunk v1) + 25+ заглушек Anchor-style команд (todo!)
    - `infrastructure/` — `hash::sha256_hex / sha256_canonical_json`, Storage trait + FolderBackend / ZipBackend (enum `AnyStorage` static dispatch), helper `open_storage` автоопределяет backend по типу файла
- [x] **Public API фасад (lib.rs)**: плоские Anchor-style вызовы (`create_db_file`, `add_object`, …) — путь к БД всегда первый аргумент
- [x] **FFI-ready**: `crate-type = [cdylib, rlib, staticlib]` — Python ctypes-интеграция (следующий шаг)
- [x] **OCC-якоря §3.2**: `manifest.state_hash` вычисляется по правилу "занулить поле → sha256 → записать обратно", `chunk.version` и `manifest.global_version` инкрементятся монотонно при каждой мутации
- [x] **25/25 integration+unit тестов** зелёные:
    - 2 unit (initializer: genesis bounds, hash format)
    - 10 create_db_file: file tree, manifest hashes, default 1e6 cube, custom 10e6 cube, theme struct parse, schema.json present, ZIP build, rejects size 0/1, overwrite folder, overwrite zip
    - 13 node_crud: add×4, delete×4, update×4, ZIP roundtrip

---

## 🚀 Ключевые архитектурные принципы

1. **Настраиваемая детерминированная система координат**: Использует нативные целые числа `i64` в сочетании с настраиваемыми глобальными и локальными масштабами единиц (например, миллиметры для архитектуры, нанометры для микрочипов) для полного устранения дрейфа точности с плавающей запятой.
2. **Контейнерный формат (`.aagl`)**: Лёгкий контейнерный формат на основе ZIP, который сбалансирует быстрый доступ во время выполнения (через выборочную загрузку чанков) с чистой системой контроля версий. Для разработки доступен Folder-режим (см. константу `CREATE_AS_FOLDER` в `common/constants.rs`).
3. **Оптимистичный контроль конкурентности (OCC)**: Встроенные хэши версий (`state_hash` / `base_version`) в манифесте для обеспечения безопасных параллельных модификаций без конфликтов со стороны нескольких ИИ-агентов.
4. **Универсальный граф (Universal Graph)**: Гибкая базовая структура для общего представления пространственных данных.
5. **Высокопроизводительное ядро на Rust**: Управление состоянием, хранение, пространственное индексирование и атомарные патчи обрабатываются сверхбыстрым движком на Rust в качестве независимой СУБД.

---

## 🏗️ Структура Rust core (`/core/`)

Код СУБД живёт в `/core/` (отдельный crate `aagl-core`). Разбит по Clean Architecture.

```text
core/src/
├── lib.rs                         # FACADE ONLY — модули + flat re-exports + thin wrappers create_db_file*
├── common/                        # Корень зависимостей — константы + ошибки (0 импортв других слоёв!)
│   ├── constants.rs               # SPEC_VERSION, CREATE_AS_FOLDER, DEFAULT_GENESIS_SPACE_SIZE…
│   └── error.rs                   # AaglError enum + pub type Result<T> (thiserror 8 variants)
├── domain/                        # PURE DATA MODELS. Ноль impl блоков, ноль IO
│   ├── manifest.rs                # Manifest, Profile, CoreHints, Units, ChunkMetadata, ViewsRegistry
│   ├── graph.rs                   # Chunk, Node, Relation (4 поля Node: parent/coords/relations/metadata)
│   └── view.rs                    # Theme (3-tier fallback) + PatchContext (OCC base anchors)
├── application/                   # Use cases / interactors
│   ├── initializer.rs             # create_empty_project (build tempdir → rename folder / zip)
│   ├── node_manager.rs            # ⭐ working v1: add_object / delete_object / update_object
│   └── commands.rs                # 25+ anchor-style signatures todo!() for next milestones
└── infrastructure/                # IO детали / driven adapters
    ├── hash.rs                    # sha256_hex("sha256:"+64hex) + sha256_canonical_json (§3.2)
    ├── storage.rs                 # StorageBackend trait + FolderBackend + ZipBackend + enum AnyStorage + zip_folder()
    └── open_storage.rs            # pub fn open_storage(path) → auto Folder vs ZIP

core/tests/
├── create_db_file_tests.rs        # 10 integration tests initializer
└── node_crud_tests.rs             # 13 integration tests node CRUD v1
```

---

## 🧪 Быстрый старт (Rust side)

```bash
# 0. Перейти в папку core/
cd core

# 1. Проверка сборки
cargo check                       # ✅ 0 errors, 0 warnings

# 2. Запустить ВСЕ тесты
cargo test                        # ✅ 25/25 passed (2 unit + 23 integration)

# 3. Запустить live-демо CRUD
cargo run
# → создаёт core/demo_crud_objects/ :
#    create → add wall → add window → update wall (stretch 4.5→6m) → delete window
#    versions: manifest.global_version=4, chunk_genesis.version=4, state_hash рекомпилирован

# 4. Собрать FFI-бандл (.dylib / .so / .dll) для Python ctypes
cargo build --release             # → target/release/libaagl_core.{dylib,so,dll}
```

---

## 🐍 Python интеграция (следующий шаг — почти готово!)

Ядро уже готово к вызову из Python через стандартный `ctypes`:
- crate-type: `cdylib` (shared lib для динамической загрузки)
- все public методы имеют Anchor-style сигнатуру "путь БД первый аргумент, возврат unified Result"
- планируемый слой: модуль `src/ffi.rs` с `extern "C"` обёртками, аргументы/результаты — JSON строки через `CString` (caller делает `aagl_free_c_string()` для предотвращения утечек, стандартный rocksdb/libgit2 паттерн)

Код примера Python-side (после сборки `--release`):
```python
from aagl_core_ffi import AaglFFI
ffi = AaglFFI(library_path="./core/target/release/libaagl_core.dylib")

ffi.create_db_file("./my_project.aagl", "Мой проект САПР")
wall_id = ffi.add_object("./my_project.aagl",
    parent_id="node_0001",
    coords=[0, 0, 0, 4500, 380, 2700],
    metadata={"profile_key":"wall", "material_code":"brick", "thickness_mm":380},
    relations=[], prefix="wall_")
ffi.update_object("./my_project.aagl", wall_id,
    new_coords=[0,0,0, 6000, 380, 2700])
ffi.delete_object("./my_project.aagl", wall_id)
```

---

## 📁 Структура репозитория

```text
aagl/
├── SPECIFICATION_en.md       # Техническая спецификация (англ.)
├── SPECIFICATION_ru.md       # Техническая спецификация (рус.)
├── example_cad_floorplan/    # Пример проекта .aagl (arch_floorplan_v0.1 — комната 4×5 м со стенами/окном/дверью)
│   ├── manifest.json         # Таблица маршрутизации + объявление профиля + якоря OCC
│   ├── schema.json           # *Связанная схема (bundle)*: (A) структурные правила Контейнерного слоя AAGL
│   │                         #                        (B) структурные правила механизма тем оформления
│   │                         #                        (C) семантические контракты metadata профиля arch_floorplan_v0.1
│   ├── chunks/chunk_genesis.json
│   └── views/default_theme.json
│
├── core/                     # ⭐ Rust crate aagl-core — ДВИЖОК СУБД (MVP работает, 25/25 tests green)
│   ├── Cargo.toml            # crate-type = [cdylib, rlib, staticlib], features = ["ffi"]
│   ├── src/                  # Clean Architecture 4 слоя (common/domain/application/infrastructure)
│   ├── tests/                # integration tests (create_db_file + node_crud = 23 test cases)
│   └── examples/             # (скоро) FFI demo examples
│
├── crates/                   # (будущее) Rust workspace — aagl-container / aagl-cli
├── bindings/                 # (следующий релиз) Python — FFI loader ctypes + pip package
└── tests/                    # (будущее) Многоагентные OCC интеграционные сценарии
```

---

📦 Формат контейнера .aagl

Вместо громоздких монолитных файлов или неуправляемых деревьев папок проект .aagl представляет собой единый портативный контейнер:

```text
project.aagl (ZIP-контейнер)  ИЛИ  project_folder/  (plain folder dev-режим, CREATE_AS_FOLDER=true)
├── manifest.json             # Глобальная таблица маршрутизации, spatial_index, хэши версий и базовые единицы
├── schema.json               # *Связанная схема (bundle)* активного профиля: структура контейнера + метаданные профиля
├── chunks/                   # Шардированные подграфы (чанки 50-100 КБ для эффективного контекста LLM)
│   ├── chunk_genesis.json    # Обязательный корневой чанк с глобальными границами (bounds) v1: ЕДИНСТВЕННЫЙ чанк
│   └── sector_01.json        # (будущее) шардирование
├── views/                    # Стратегии презентации (темы стилей — не входят в OCC-хэши данных)
│   └── default_theme.json
└── assets/                   # Бинарные ресурсы (текстуры, сканы планов, облака точек)
```

---

🛠️ Технологический стек

Ядро СУБД: Rust (абстракции с нулевой стоимостью, безопасность памяти, высокоскоростной парсинг и пространственная индексация).
- `serde / serde_json` — сериализация, канонический JSON для хешей
- `thiserror` — единый enum ошибок AaglError
- `zip` (deflate) — ZIP-бэкенд `.aagl`
- `sha2` — sha256 хеши OCC
- `tempfile` — атомарная запись (tempdir → переименование)

Обмен данными: JSON Patch (RFC 6902) для атомарных обновлений с сохранением состояния.

Python-интеграция: стандартный `ctypes` + строковые JSON-аргументы (zero extra dependencies, работает на любой CPython 3.8+ без PyO3 build).

---

## 📄 Лицензия

AAGL использует модель двойного лицензирования:

Некоммерческое использование: Этот проект лицензирован по Некоммерческой лицензии — свободно для академических, исследовательских, личных целей и целей оценки.

Коммерческое использование: Для любых коммерческих приложений, корпоративного развёртывания или интеграции в продукты, приносящие доход, требуется Коммерческая лицензия.

Подробности см. в файле LICENSE или свяжитесь с нами по вопросам коммерческого использования.

---

## 🗺️ Дорожная карта

- [x] **v0.1.0 Core MVP**: Clean Architecture 4 слоя, create_db_file, dual Folder/ZIP backend, genesis cube, OCC anchors (state_hash + versions), 3 node CRUD operations single-chunk, 25/25 tests green, FFI-ready cdylib build target
- [ ] **v0.1.1**: Python ctypes FFI бандл + pip package (`pip install aagl-core`) + `demo_crud.py`
- [ ] **v0.2**: Multi-chunk graph (`register_chunk`, `load_chunk`, `spatial_index` in manifest)
- [ ] **v0.3**: OCC real validation (`verify_occ(ctx)`, apply_data_patch with conflict returns Err OccConflict)
- [ ] **v0.4**: Profile Schema Layer (arch_cad_v1): TypeDefinitions for wall/window/door/column/beam, compile schema.json, coords layout validation (bbox6/polyline2N/…)
- [ ] **v0.5**: Пространственное индексирование (R-Tree) и многоагентные OCC тесты
- [ ] **v0.6**: Упаковщик/распаковщик `aagl-cli` (folder ↔ .aagl) + git hooks helper
- [ ] **v1.0-beta**: Расширяемые правила валидации профилей (JSON Schema) + стабильный FFI API ABI stable
