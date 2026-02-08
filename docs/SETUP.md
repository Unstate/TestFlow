# TestFlow - Локальная установка и запуск

## Требования

- **Rust** 1.85+ (edition 2024) — https://rustup.rs/
- **PostgreSQL** 15+ — https://www.postgresql.org/download/
- **Git** (опционально)

## 1. Установка Rust

Если Rust ещё не установлен:

```bash
# Windows: скачайте и запустите rustup-init.exe с https://rustup.rs/
# Linux/macOS:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Проверка:
```bash
rustc --version
cargo --version
```

## 2. Установка и настройка PostgreSQL

### Windows
1. Скачайте установщик с https://www.postgresql.org/download/windows/
2. Установите с паролем по умолчанию `postgres`

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

### macOS
```bash
brew install postgresql@15
brew services start postgresql@15
```

## 3. Создание базы данных

```bash
# Подключитесь к PostgreSQL
psql -U postgres

# Создайте базу данных
CREATE DATABASE testflow;

# Выход
\q
```

## 4. Настройка окружения

Скопируйте файл `.env.example` в `.env` (или отредактируйте существующий `.env`):

```bash
cp .env.example .env
```

Отредактируйте `.env` при необходимости:
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/testflow
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRATION_HOURS=24
RUST_LOG=testflow=debug,tower_http=debug
```

**Важно:** Замените `postgres:postgres` на ваш логин и пароль PostgreSQL, если они отличаются.

## 5. Сборка и запуск

```bash
# Перейдите в директорию проекта
cd TestFLow

# Сборка проекта
cargo build

# Запуск
cargo run
```

При первом запуске:
- Автоматически применятся миграции (создание таблиц)
- Автоматически создастся администратор по умолчанию:
  - **Логин:** `admin`
  - **Пароль:** `admin123`

Сервер запустится на `http://localhost:3000`.

## 6. Проверка работоспособности

Откройте в браузере:
- **Swagger UI:** http://localhost:3000/swagger-ui/
- **OpenAPI JSON:** http://localhost:3000/api-docs/openapi.json

## Структура проекта

```
TestFLow/
├── Cargo.toml              # Зависимости проекта
├── .env                    # Переменные окружения
├── migrations/
│   └── 001_init.sql        # SQL-миграция (таблицы, типы, индексы)
├── src/
│   ├── main.rs             # Точка входа, роутер, Swagger
│   ├── config.rs           # Конфигурация БД и приложения
│   ├── auth.rs             # JWT аутентификация
│   ├── models.rs           # Модели данных (User, Task, enums)
│   ├── dto.rs              # Data Transfer Objects
│   ├── errors.rs           # Обработка ошибок
│   └── handlers/
│       ├── mod.rs
│       ├── auth_handler.rs # Эндпоинт логина
│       ├── user_handler.rs # CRUD пользователей
│       └── task_handler.rs # CRUD задач + статистика
└── docs/
    ├── backend_development_plan.md
    └── project_documentation.md
```

## Роли в системе

| Роль       | Описание                                     |
|------------|----------------------------------------------|
| admin      | Управление пользователями. Не может работать с задачами |
| manager    | Распределение задач, просмотр статистики     |
| tester     | Выполнение тестирования                      |
| developer  | Постановка задач на тестирование             |

## API эндпоинты

| Метод  | Путь                        | Описание                        | Доступ        |
|--------|-----------------------------|---------------------------------|---------------|
| POST   | /api/auth/login             | Авторизация                     | Все           |
| GET    | /api/users/me               | Текущий пользователь            | Авторизованные|
| GET    | /api/users                  | Список пользователей            | admin         |
| GET    | /api/users/{id}             | Пользователь по ID              | admin         |
| POST   | /api/users                  | Создать пользователя            | admin         |
| PUT    | /api/users/{id}             | Обновить пользователя           | admin         |
| DELETE | /api/users/{id}             | Удалить пользователя            | admin         |
| GET    | /api/tasks                  | Список задач (с фильтрами)     | Авторизованные|
| GET    | /api/tasks/{id}             | Задача по ID                    | Авторизованные|
| POST   | /api/tasks                  | Создать задачу                  | Все кроме admin|
| PUT    | /api/tasks/{id}             | Обновить задачу                 | Все кроме admin|
| DELETE | /api/tasks/{id}             | Удалить задачу                  | Создатель/manager|
| GET    | /api/statistics/employees   | Статистика по сотрудникам       | manager/admin |
