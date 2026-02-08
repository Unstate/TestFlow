# TestFlow - Руководство по тестированию API

## Подготовка

1. Убедитесь, что сервер запущен (`cargo run`)
2. Сервер доступен на http://localhost:3000
3. Swagger UI: http://localhost:3000/swagger-ui/

Все примеры ниже используют `curl`. Можно использовать Postman, Insomnia или Swagger UI.

---

## 1. Авторизация

### 1.1 Логин администратора (создаётся автоматически)

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

Ответ:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "user": {
    "id": "...",
    "username": "admin",
    "role": "admin",
    ...
  }
}
```

**Сохраните `token` — он нужен для всех последующих запросов.**

В примерах ниже заменяйте `<TOKEN>` на полученный JWT-токен.

---

## 2. Управление пользователями (только admin)

### 2.1 Создать пользователя-менеджера

```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{
    "username": "manager1",
    "email": "manager1@testflow.local",
    "password": "password123",
    "full_name": "Иван Петров",
    "role": "manager"
  }'
```

### 2.2 Создать тестировщика

```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{
    "username": "tester1",
    "email": "tester1@testflow.local",
    "password": "password123",
    "full_name": "Анна Сидорова",
    "role": "tester"
  }'
```

### 2.3 Создать разработчика

```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{
    "username": "developer1",
    "email": "dev1@testflow.local",
    "password": "password123",
    "full_name": "Олег Платов",
    "role": "developer"
  }'
```

### 2.4 Получить список всех пользователей

```bash
curl -X GET http://localhost:3000/api/users \
  -H "Authorization: Bearer <TOKEN>"
```

### 2.5 Получить пользователя по ID

```bash
curl -X GET http://localhost:3000/api/users/<USER_ID> \
  -H "Authorization: Bearer <TOKEN>"
```

### 2.6 Обновить пользователя

```bash
curl -X PUT http://localhost:3000/api/users/<USER_ID> \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{
    "full_name": "Иван Петрович Петров",
    "is_active": true
  }'
```

### 2.7 Удалить пользователя

```bash
curl -X DELETE http://localhost:3000/api/users/<USER_ID> \
  -H "Authorization: Bearer <TOKEN>"
```

### 2.8 Проверка запрета: не-админ пытается создать пользователя

```bash
# Сначала залогиньтесь как developer1
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "developer1", "password": "password123"}'

# Попытка создать пользователя (должна вернуть 403)
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <DEV_TOKEN>" \
  -d '{
    "username": "hacker",
    "email": "hack@test.com",
    "password": "123456",
    "full_name": "Хакер",
    "role": "admin"
  }'
```

Ожидаемый ответ: `403 Forbidden`

---

## 3. Управление задачами

### 3.1 Создать задачу (от имени разработчика)

Сначала залогиньтесь как `developer1` и получите токен.

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <DEV_TOKEN>" \
  -d '{
    "title": "Протестировать модуль авторизации",
    "description": "Необходимо провести функциональное тестирование JWT авторизации",
    "tester_id": "<TESTER_USER_ID>",
    "urgency": "high",
    "acceptance_criteria": "Все тесты пройдены, нет критических багов",
    "evaluation_criteria": "Покрытие всех сценариев из ТЗ",
    "comment": "Ссылка на ТЗ: https://docs.example.com/auth-spec"
  }'
```

### 3.2 Проверка: админ не может создать задачу

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -d '{
    "title": "Тестовая задача",
    "description": "Описание"
  }'
```

Ожидаемый ответ: `403 Forbidden` — "Administrators cannot create tasks"

### 3.3 Получить список задач

```bash
curl -X GET http://localhost:3000/api/tasks \
  -H "Authorization: Bearer <TOKEN>"
```

### 3.4 Получить список задач с фильтрами

```bash
# По статусу
curl -X GET "http://localhost:3000/api/tasks?status=new" \
  -H "Authorization: Bearer <TOKEN>"

# По срочности
curl -X GET "http://localhost:3000/api/tasks?urgency=high" \
  -H "Authorization: Bearer <TOKEN>"

# По тестировщику
curl -X GET "http://localhost:3000/api/tasks?tester_id=<TESTER_ID>" \
  -H "Authorization: Bearer <TOKEN>"

# Пагинация
curl -X GET "http://localhost:3000/api/tasks?page=1&per_page=10" \
  -H "Authorization: Bearer <TOKEN>"
```

### 3.5 Получить задачу по ID

```bash
curl -X GET http://localhost:3000/api/tasks/<TASK_ID> \
  -H "Authorization: Bearer <TOKEN>"
```

### 3.6 Обновить задачу (изменить статус)

```bash
curl -X PUT http://localhost:3000/api/tasks/<TASK_ID> \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <DEV_TOKEN>" \
  -d '{
    "status": "in_progress",
    "comment": "Начато тестирование"
  }'
```

### 3.7 Закрыть задачу

```bash
curl -X PUT http://localhost:3000/api/tasks/<TASK_ID> \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <DEV_TOKEN>" \
  -d '{
    "status": "closed"
  }'
```

При закрытии автоматически проставляется `closed_at`.

### 3.8 Удалить задачу

```bash
curl -X DELETE http://localhost:3000/api/tasks/<TASK_ID> \
  -H "Authorization: Bearer <DEV_TOKEN>"
```

---

## 4. Статистика (manager/admin)

### 4.1 Статистика по сотрудникам

```bash
curl -X GET http://localhost:3000/api/statistics/employees \
  -H "Authorization: Bearer <MANAGER_TOKEN>"
```

Ответ:
```json
[
  {
    "user_id": "...",
    "full_name": "Анна Сидорова",
    "total_tasks": 5,
    "completed_tasks": 2,
    "in_progress_tasks": 1
  }
]
```

---

## 5. Профиль текущего пользователя

```bash
curl -X GET http://localhost:3000/api/users/me \
  -H "Authorization: Bearer <TOKEN>"
```

---

## 6. Тестирование через Swagger UI

1. Откройте http://localhost:3000/swagger-ui/
2. Нажмите кнопку **Authorize** в правом верхнем углу
3. Введите JWT-токен (без префикса `Bearer `)
4. Нажмите **Authorize**
5. Теперь все запросы будут автоматически содержать токен
6. Выберите любой эндпоинт, нажмите **Try it out**, заполните параметры и нажмите **Execute**

---

## 7. Типичный сценарий тестирования

1. Залогиниться как `admin` -> получить токен
2. Создать пользователей (manager, tester, developer)
3. Залогиниться как `developer1` -> получить токен
4. Создать задачу от имени developer1, назначить tester1
5. Залогиниться как `tester1`
6. Просмотреть список задач
7. Обновить статус задачи на `in_progress`
8. Обновить статус на `done`
9. Залогиниться как `manager1`
10. Просмотреть статистику по сотрудникам
11. Отфильтровать задачи по статусу/срочности

---

## 8. Коды ошибок

| Код | Описание                              |
|-----|---------------------------------------|
| 200 | Успешный запрос                       |
| 201 | Ресурс создан                         |
| 204 | Ресурс удалён                         |
| 400 | Ошибка валидации                      |
| 401 | Не авторизован / неверный токен       |
| 403 | Доступ запрещён (недостаточно прав)   |
| 404 | Ресурс не найден                      |
| 409 | Конфликт (дублирование username/email)|
| 500 | Внутренняя ошибка сервера             |
