# Work Item API

This module provides a REST API and HTML form interface for adding work items to the database.

## Setup

### Database Configuration
The API is configured to use a SQLite database file named `workstat.db`.

## Running the API Server

To start the API server, from the main directory run:

```bash
cd workstat-rs && cargo build && cd ..
./workstat-rs/target/debug/main
```

This will start a development server on `http://localhost:5004`.

## Using the API

### HTML Form Interface
Access the form interface by navigating to:
```
http://localhost:5004/
```

This provides a user-friendly form to submit new work items.

### REST API Endpoint

#### Add a Work Item
- **URL**: `/api/work-insert`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
  ```json
  {
    "uname_o": "original_username",
    "uname_u": "upstream_username",
    "tdiff": 123,
    "sec": "secret_value"
  }
  ```

- **Success Response**:
  - **Code**: 201 Created
  - **Content**:
    ```json
    {
      "message": "Work item added successfully"
    }
    ```

- **Error Response**:
  - **Code**: 400 Bad Request
  - **Content**:
    ```json
    {
      "error": "Missing required field: tdiff"
    }
    ```
  OR
  - **Code**: 500 Internal Server Error
  - **Content**:
    ```json
    {
      "error": "Error message description"
    }
    ```

## Example Usage

### Using curl

```bash
curl -X POST http://localhost:5004/api/work-insert \
  -H "Content-Type: application/json" \
  -d '{"uname_o": "user1", "uname_u": "upstream1", "tdiff": 100, "sec": "secret_value", "pool": 0}'
```

### Using Python requests

```python
import requests
import json

url = "http://localhost:5000/-insert"
data = {
    "uname_o": "user1",
    "uname_u": "upstream1",
    "tdiff": 100,
    "pool": 0,
}
headers = {"Content-Type": "application/json"}

response = requests.post(url, data=json.dumps(data), headers=headers)
print(response.json())
