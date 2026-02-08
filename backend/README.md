# TestFlow Backend

REST API для системы управления задачами и тестированием, построенный на Rust с использованием Axum.

## Технологический стек

- **Rust** (edition 2024) + **Axum 0.8** — веб-фреймворк
- **SQLx 0.8** — асинхронное взаимодействие с PostgreSQL
- **JWT** (`jsonwebtoken`) — аутентификация
- **Argon2** — хеширование паролей
- **utoipa** + **Swagger UI** — автогенерация OpenAPI-документации
- **validator** — валидация входных данных

## Быстрый старт

### Требования

- Rust (stable)
- PostgreSQL 15+

### Установка и запуск

1. Создайте базу данных:

```sql
CREATE DATABASE testflow;
```

2. Настройте переменные окружения:

```bash
cp .env.example .env
# Отредактируйте .env, указав свои параметры подключения к БД
```

3. Запустите сервер:

```bash
cargo run
```

Сервер запустится на `http://localhost:3000`. Миграции применяются автоматически при старте.

### Учетная запись по умолчанию

При первом запуске (если в БД нет пользователей) создается администратор:

- **Логин:** `admin`
- **Пароль:** `admin123`

## Переменные окружения

| Переменная             | Обязательная | По умолчанию | Описание                         |
|------------------------|:------------:|:------------:|----------------------------------|
| `DATABASE_URL`         | да           | —            | Строка подключения к PostgreSQL  |
| `JWT_SECRET`           | да           | —            | Секретный ключ для подписи JWT   |
| `JWT_EXPIRATION_HOURS` | нет          | `24`         | Время жизни токена (в часах)     |
| `RUST_LOG`             | нет          | `testflow=debug,tower_http=debug` | Уровень логирования |

## API

### Swagger UI

После запуска документация доступна по адресу: `http://localhost:3000/swagger-ui/`

OpenAPI JSON: `http://localhost:3000/api-docs/openapi.json`

### Эндпоинты

#### Аутентификация

| Метод | Путь              | Описание         | Доступ     |
|-------|-------------------|------------------|------------|
| POST  | `/api/auth/login` | Вход в систему   | Все        |

#### Пользователи

| Метод  | Путь              | Описание                    | Доступ |
|--------|-------------------|-----------------------------|--------|
| GET    | `/api/users`      | Список пользователей        | Admin  |
| GET    | `/api/users/me`   | Текущий пользователь        | Все    |
| GET    | `/api/users/{id}` | Получить пользователя по ID | Admin  |
| POST   | `/api/users`      | Создать пользователя        | Admin  |
| PUT    | `/api/users/{id}` | Обновить пользователя       | Admin  |
| DELETE | `/api/users/{id}` | Удалить пользователя        | Admin  |

#### Задачи

| Метод  | Путь             | Описание                 | Доступ              |
|--------|------------------|--------------------------|---------------------|
| GET    | `/api/tasks`     | Список задач (фильтры)   | Все авторизованные  |
| GET    | `/api/tasks/{id}`| Получить задачу по ID    | Все авторизованные  |
| POST   | `/api/tasks`     | Создать задачу           | Manager, Developer, Tester |
| PUT    | `/api/tasks/{id}`| Обновить задачу          | Manager, Developer, Tester |
| DELETE | `/api/tasks/{id}`| Удалить задачу           | Создатель / Manager |

#### Статистика

| Метод | Путь                         | Описание              | Доступ         |
|-------|------------------------------|-----------------------|----------------|
| GET   | `/api/statistics/employees`  | Статистика сотрудников| Admin, Manager |

### Фильтрация задач

GET `/api/tasks` поддерживает query-параметры:

- `status` — фильтр по статусу (`new`, `in_progress`, `testing`, `done`, `closed`)
- `urgency` — фильтр по срочности (`low`, `medium`, `high`, `critical`)
- `tester_id` — UUID тестировщика
- `assigned_by` — UUID автора задачи
- `page` — номер страницы (по умолчанию `1`)
- `per_page` — количество на странице (по умолчанию `20`)

## Модель данных

### Роли пользователей

| Роль        | Описание              |
|-------------|-----------------------|
| `admin`     | Системный администратор — управление пользователями |
| `manager`   | Менеджер — управление задачами и просмотр статистики |
| `tester`    | Тестировщик           |
| `developer` | Разработчик           |

### Статусы задач

`new` → `in_progress` → `testing` → `done` → `closed`

### Срочность задач

`low` | `medium` | `high` | `critical`

## Структура проекта

```
backend/
├── src/
│   ├── main.rs          # Точка входа, маршруты, миграции
│   ├── config.rs        # Конфигурация и пул БД
│   ├── models.rs        # Модели данных (User, Task, enum'ы)
│   ├── dto.rs           # DTO для запросов и ответов
│   ├── errors.rs        # Обработка ошибок
│   ├── auth.rs          # JWT и AuthUser extractor
│   └── handlers/
│       ├── mod.rs
│       ├── auth_handler.rs  # POST /api/auth/login
│       ├── user_handler.rs  # CRUD пользователей
│       └── task_handler.rs  # CRUD задач, статистика
├── migrations/
│   └── 001_init.sql     # Начальная схема БД
├── docs/                # Документация проекта
├── Cargo.toml
├── .env.example
└── .env
```

## Примеры использования (cURL)

### Авторизация

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

### Создание пользователя (admin)

```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "username": "developer1",
    "email": "dev@testflow.local",
    "password": "password123",
    "full_name": "Ivan Ivanov",
    "role": "developer"
  }'
```

### Создание задачи

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "title": "Написать unit-тесты для модуля авторизации",
    "description": "Покрыть тестами auth.rs",
    "urgency": "high"
  }'
```
