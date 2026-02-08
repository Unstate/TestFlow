# План разработки Backend (Rust + Rocket + PostgreSQL)

## Технологический стек

| Компонент | Технология | Версия |
|-----------|------------|--------|
| Язык | Rust | 1.75+ |
| Web Framework | Rocket | 0.5 |
| ORM | Diesel | 2.1 |
| База данных | PostgreSQL | 15+ |
| Аутентификация | JWT (jsonwebtoken) | 9.x |
| Хеширование паролей | Argon2 (argon2) | 0.5 |
| Сериализация | Serde | 1.x |
| Валидация | Validator | 0.16 |
| Миграции | Diesel CLI | 2.1 |
| Логирование | tracing | 0.1 |
| Тестирование | cargo test + rstest | - |

---

## Структура проекта

```
testflow-backend/
├── Cargo.toml
├── Cargo.lock
├── Rocket.toml                 # Конфигурация Rocket
├── .env                        # Переменные окружения
├── .env.example
├── diesel.toml                 # Конфигурация Diesel
│
├── migrations/                 # Миграции БД
│   ├── 00000000000001_create_users/
│   │   ├── up.sql
│   │   └── down.sql
│   ├── 00000000000002_create_roles/
│   └── ...
│
├── src/
│   ├── main.rs                 # Точка входа
│   ├── lib.rs                  # Экспорт модулей
│   │
│   ├── config/                 # Конфигурация
│   │   ├── mod.rs
│   │   └── database.rs
│   │
│   ├── schema.rs               # Diesel schema (автогенерация)
│   │
│   ├── models/                 # Модели данных (Diesel)
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── role.rs
│   │   ├── test_request.rs
│   │   ├── task.rs
│   │   ├── test_suite.rs
│   │   ├── test_case.rs
│   │   ├── test_data.rs
│   │   ├── test_result.rs
│   │   ├── notification.rs
│   │   └── report.rs
│   │
│   ├── dto/                    # Data Transfer Objects
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── user.rs
│   │   ├── test_request.rs
│   │   ├── task.rs
│   │   └── statistics.rs
│   │
│   ├── repositories/           # Слой доступа к данным
│   │   ├── mod.rs
│   │   ├── user_repository.rs
│   │   ├── test_request_repository.rs
│   │   ├── task_repository.rs
│   │   └── ...
│   │
│   ├── services/               # Бизнес-логика
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── user_service.rs
│   │   ├── test_request_service.rs
│   │   ├── task_service.rs
│   │   ├── notification_service.rs
│   │   ├── statistics_service.rs
│   │   └── file_service.rs
│   │
│   ├── routes/                 # HTTP обработчики (Rocket routes)
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── test_requests.rs
│   │   ├── tasks.rs
│   │   ├── test_suites.rs
│   │   ├── notifications.rs
│   │   └── statistics.rs
│   │
│   ├── guards/                 # Rocket Guards
│   │   ├── mod.rs
│   │   ├── auth_guard.rs       # JWT валидация
│   │   └── role_guard.rs       # Проверка ролей
│   │
│   ├── middleware/             # Fairings (middleware)
│   │   ├── mod.rs
│   │   ├── cors.rs
│   │   └── logging.rs
│   │
│   ├── errors/                 # Обработка ошибок
│   │   ├── mod.rs
│   │   └── api_error.rs
│   │
│   └── utils/                  # Утилиты
│       ├── mod.rs
│       ├── jwt.rs
│       └── password.rs
│
└── tests/                      # Интеграционные тесты
    ├── common/
    │   └── mod.rs
    ├── auth_tests.rs
    ├── user_tests.rs
    └── task_tests.rs
```

---

## Схема базы данных

### ERD (Entity Relationship Diagram)

```
┌─────────────┐       ┌─────────────┐
│   roles     │       │   users     │
├─────────────┤       ├─────────────┤
│ id (PK)     │◄──────│ id (PK)     │
│ name        │       │ role_id(FK) │
│ description │       │ email       │
└─────────────┘       │ password    │
                      │ first_name  │
                      │ last_name   │
                      │ is_active   │
                      │ created_at  │
                      │ updated_at  │
                      └──────┬──────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ test_requests   │  │     tasks       │  │  notifications  │
├─────────────────┤  ├─────────────────┤  ├─────────────────┤
│ id (PK)         │  │ id (PK)         │  │ id (PK)         │
│ developer_id(FK)│  │ request_id (FK) │  │ user_id (FK)    │
│ title           │  │ assignee_id(FK) │  │ title           │
│ description     │  │ assigned_by(FK) │  │ message         │
│ status          │  │ status          │  │ is_read         │
│ created_at      │  │ deadline        │  │ created_at      │
└────────┬────────┘  │ started_at      │  └─────────────────┘
         │           │ completed_at    │
         │           └────────┬────────┘
         │                    │
         ▼                    ▼
┌─────────────────┐  ┌─────────────────┐
│  attachments    │  │  test_suites    │
├─────────────────┤  ├─────────────────┤
│ id (PK)         │  │ id (PK)         │
│ request_id (FK) │  │ task_id (FK)    │
│ filename        │  │ name            │
│ file_path       │  │ description     │
│ file_type       │  │ created_at      │
│ file_size       │  └────────┬────────┘
└─────────────────┘           │
                              ▼
                     ┌─────────────────┐
                     │   test_cases    │
                     ├─────────────────┤
                     │ id (PK)         │
                     │ suite_id (FK)   │
                     │ title           │
                     │ description     │
                     │ expected_result │
                     │ status          │
                     └────────┬────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
     ┌─────────────────┐             ┌─────────────────┐
     │ test_data_sets  │             │  test_results   │
     ├─────────────────┤             ├─────────────────┤
     │ id (PK)         │             │ id (PK)         │
     │ test_case_id(FK)│             │ test_case_id(FK)│
     │ name            │             │ status          │
     │ data (JSONB)    │             │ comment         │
     └─────────────────┘             │ executed_at     │
                                     │ executed_by(FK) │
                                     └─────────────────┘
```

---

## API Endpoints

### Аутентификация

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| POST | `/api/auth/login` | Вход в систему | Public |
| POST | `/api/auth/logout` | Выход из системы | All |
| GET | `/api/auth/me` | Текущий пользователь | All |
| POST | `/api/auth/refresh` | Обновление токена | All |

### Пользователи (Admin)

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/users` | Список пользователей | Admin |
| GET | `/api/users/:id` | Детали пользователя | Admin |
| POST | `/api/users` | Создать пользователя | Admin |
| PUT | `/api/users/:id` | Обновить пользователя | Admin |
| DELETE | `/api/users/:id` | Удалить/деактивировать | Admin |
| GET | `/api/users?role=tester` | Список по роли | Manager |

### Заявки на тестирование

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/test-requests` | Мои заявки | Developer |
| GET | `/api/test-requests/:id` | Детали заявки | Developer, Manager |
| POST | `/api/test-requests` | Создать заявку | Developer |
| POST | `/api/test-requests/:id/attachments` | Загрузить файл | Developer |
| GET | `/api/admin/test-requests` | Все заявки | Manager |

### Задачи

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/tasks` | Все задачи | Manager |
| GET | `/api/tasks/my` | Мои задачи | Tester, Manager |
| GET | `/api/tasks/:id` | Детали задачи | Tester, Manager |
| POST | `/api/tasks` | Создать задачу | Manager |
| PUT | `/api/tasks/:id/assign` | Назначить исполнителя | Manager |
| PUT | `/api/tasks/:id/deadline` | Установить дедлайн | Manager |
| PUT | `/api/tasks/:id/status` | Изменить статус | Tester, Manager |

### Тесты

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/tasks/:id/test-suites` | Наборы тестов задачи | Tester, Manager |
| POST | `/api/tasks/:id/test-suites` | Создать набор тестов | Tester, Manager |
| PUT | `/api/test-suites/:id` | Обновить набор | Tester, Manager |
| DELETE | `/api/test-suites/:id` | Удалить набор | Tester, Manager |
| GET | `/api/test-suites/:id/test-cases` | Тест-кейсы набора | Tester, Manager |
| POST | `/api/test-suites/:id/test-cases` | Создать тест-кейс | Tester, Manager |
| PUT | `/api/test-cases/:id` | Обновить тест-кейс | Tester, Manager |
| DELETE | `/api/test-cases/:id` | Удалить тест-кейс | Tester, Manager |

### Тестовые данные и результаты

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/test-cases/:id/data-sets` | Тестовые данные | Tester, Manager |
| POST | `/api/test-cases/:id/data-sets` | Добавить данные | Tester, Manager |
| PUT | `/api/data-sets/:id` | Обновить данные | Tester, Manager |
| DELETE | `/api/data-sets/:id` | Удалить данные | Tester, Manager |
| POST | `/api/test-cases/:id/results` | Записать результат | Tester, Manager |
| GET | `/api/tasks/:id/results` | Результаты задачи | Tester, Manager |

### Уведомления

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/notifications` | Мои уведомления | All |
| PUT | `/api/notifications/:id/read` | Отметить прочитанным | All |
| PUT | `/api/notifications/read-all` | Отметить все | All |

### Статистика

| Метод | Endpoint | Описание | Роли |
|-------|----------|----------|------|
| GET | `/api/statistics/tasks-by-user` | Задачи по сотрудникам | Manager |
| GET | `/api/statistics/avg-duration` | Средняя длительность | Manager |
| GET | `/api/dashboard/manager` | Данные для дашборда | Manager |
| GET | `/api/statistics/system` | Общая статистика | Admin |

---

## Миграции (порядок создания)

# Sprint 1
diesel migration generate create_roles
diesel migration generate create_users

# Sprint 2  
diesel migration generate create_test_requests
diesel migration generate create_attachments
diesel migration generate create_tasks

# Sprint 3
diesel migration generate create_test_suites
diesel migration generate create_test_cases
diesel migration generate create_test_data_sets
diesel migration generate create_test_results
diesel migration generate create_notifications

## Чек-лист разработки

### Sprint 1 Backend Tasks

- [ ] `cargo new testflow-backend`
- [ ] Настроить Cargo.toml с зависимостями
- [ ] Настроить Rocket.toml
- [ ] Настроить подключение к PostgreSQL
- [ ] Создать миграции для users и roles
- [ ] Реализовать модели User, Role
- [ ] Реализовать UserRepository
- [ ] Реализовать AuthService (login, JWT)
- [ ] Реализовать UserService (CRUD)
- [ ] Реализовать AuthGuard
- [ ] Реализовать RoleGuard
- [ ] Настроить CORS
- [ ] Написать тесты для auth endpoints
- [ ] Документировать API

### Sprint 2 Backend Tasks

- [ ] Миграции: test_requests, tasks, attachments
- [ ] Модели: TestRequest, Task, Attachment
- [ ] TestRequestRepository, TaskRepository
- [ ] TestRequestService, TaskService
- [ ] FileService для загрузки файлов
- [ ] Routes для test_requests и tasks
- [ ] Dashboard endpoints
- [ ] Тесты

### Sprint 3 Backend Tasks

- [ ] Миграции: test_suites, test_cases, test_data, results, notifications
- [ ] Все оставшиеся модели
- [ ] NotificationService
- [ ] StatisticsService
- [ ] Все оставшиеся routes
- [ ] Генерация отчётов
- [ ] Интеграционные тесты

---
