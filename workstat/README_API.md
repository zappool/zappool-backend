# Work Item API

This module provides a REST API and HTML form interface for adding work items to the database.

## Setup

### Prerequisites
- Python 3.6+
- Flask (install with `pip install flask`)

### Database Configuration
The API is configured to use a SQLite database file named `workstat.db` by default. If your database has a different name, update the `DATABASE` variable in `api.py`.

## Running the API Server

To start the API server, navigate to the `workstat/src` directory and run:

```bash
python api.py
```

This will start a development server on `http://localhost:5000`.

## Using the API

### HTML Form Interface
Access the form interface by navigating to:
```
http://localhost:5000/
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
    "tdiff": 123
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
curl -X POST http://localhost:5000/api/work-insert \
  -H "Content-Type: application/json" \
  -d '{"uname_o": "user1", "uname_u": "upstream1", "tdiff": 100}'
```

### Using Python requests

```python
import requests
import json

url = "http://localhost:5000/-insert"
data = {
    "uname_o": "user1",
    "uname_u": "upstream1",
    "tdiff": 100
}
headers = {"Content-Type": "application/json"}

response = requests.post(url, data=json.dumps(data), headers=headers)
print(response.json())
