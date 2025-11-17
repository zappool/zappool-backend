import db_ws

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

import dotenv
from flask import Flask, request, jsonify, render_template_string
import sqlite3
import os
import sys


dotenv.load_dotenv()
API_SECRET = os.getenv("WORKSTAT_SECRET")
if not API_SECRET or len(API_SECRET) < 2:
    print(f"Error: API_SECRET is unset or too weak '{API_SECRET}'")
    sys.exit(-1)
print(f"Using Api secret, len {len(API_SECRET)}")

app = Flask(__name__)

# Database configuration
DATABASE = "workstat.db"  # Database filename (adjust if needed)
dbfile = get_db_file(DATABASE)

def get_db_connection(readonly: bool):
    """Create a connection to the SQLite database."""
    global dbfile
    if readonly:
        dbfile_uri_ro = "file:" + dbfile + "?mode=ro"
        # print(dbfile_uri_ro)
        conn = sqlite3.connect(dbfile_uri_ro, uri=True)
    else:
        conn = sqlite3.connect(dbfile)
    return conn

# HTML template for the form
HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <title>Work Item Submission</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }
        .form-group {
            margin-bottom: 15px;
        }
        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }
        input {
            width: 100%;
            padding: 8px;
            box-sizing: border-box;
        }
        button {
            background-color: #4CAF50;
            color: white;
            padding: 10px 15px;
            border: none;
            cursor: pointer;
        }
        .error {
            color: red;
            margin-top: 10px;
        }
        .success {
            color: green;
            margin-top: 10px;
        }
    </style>
</head>
<body>
    <h1>Submit New Work Item</h1>
    <div id="message"></div>
    
    <form id="workForm">
        <div class="form-group">
            <label for="uname_o">Original Username:</label>
            <input type="text" id="uname_o" name="uname_o" required>
        </div>
        
        <div class="form-group">
            <label for="uname_u">Upstream Username:</label>
            <input type="text" id="uname_u" name="uname_u" required>
        </div>
        
        <div class="form-group">
            <label for="tdiff">Target Difficulty:</label>
            <input type="number" id="tdiff" name="tdiff" required min="1">
        </div>
        
        <button type="submit">Submit</button>
    </form>
    
    <script>
        document.getElementById('workForm').addEventListener('submit', function(e) {
            e.preventDefault();
            
            const formData = {
                uname_o: document.getElementById('uname_o').value,
                uname_u: document.getElementById('uname_u').value,
                tdiff: parseInt(document.getElementById('tdiff').value)
            };
            
            fetch('/api/work-insert', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(formData),
            })
            .then(response => response.json())
            .then(data => {
                const messageDiv = document.getElementById('message');
                if (data.error) {
                    messageDiv.className = 'error';
                    messageDiv.textContent = 'Error: ' + data.error;
                } else {
                    messageDiv.className = 'success';
                    messageDiv.textContent = 'Work item successfully added!';
                    document.getElementById('workForm').reset();
                }
            })
            .catch(error => {
                const messageDiv = document.getElementById('message');
                messageDiv.className = 'error';
                messageDiv.textContent = 'Error submitting form: ' + error.message;
            });
        });
    </script>
</body>
</html>
"""

@app.route('/')
def index():
    """Serve the HTML form for submitting work items."""
    return render_template_string(HTML_TEMPLATE)

@app.route('/api/ping', methods=['GET'])
def ping():
    try:
        print(f"Received ping")
        return jsonify({"pong": "ok"}), 200
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/api/work-insert', methods=['POST'])
def add_work():
    """API endpoint to add a new work item to the database."""
    try:
        # Get JSON data from request
        data = request.get_json()
        
        # Validate required fields
        if not data:
            return jsonify({"error": "No data provided"}), 400
            
        # Extract and validate fields
        required_fields = ['uname_o', 'uname_u', 'tdiff']
        for field in required_fields:
            if field not in data:
                return jsonify({"error": f"Missing required field: {field}"}), 400
        
        uname_o = data['uname_o']
        uname_u = data['uname_u']
        
        # Validate tdiff is an integer
        try:
            tdiff = int(data['tdiff'])
            if tdiff <= 0:
                return jsonify({"error": "Target difficulty must be a positive integer"}), 400
        except (ValueError, TypeError):
            return jsonify({"error": "Target difficulty must be an integer"}), 400

        secret = data['sec']

        print(f"Received work: '{uname_o}' '{uname_u}' {tdiff}")

        # Validate secret
        if secret != API_SECRET:
            print(f"Wrong API secret received! ({secret}) (expected secret from .env)")
            return jsonify({"error": f"Incorrect API secret!"}), 500

        # Insert work item into database
        try:
            conn = get_db_connection(readonly=False)
            db_ws.insert_work(conn, uname_o, uname_u, tdiff)
            conn.close()
        except Exception as e:
            return jsonify({"error": f"Error inserting {str(e)}"}), 500
        
        return jsonify({"message": "Work item added successfully"}), 201
        
    except Exception as e:
        return jsonify({"error": str(e)}), 500

# API endpoint for form submissions
@app.route('/api/work-insert/form', methods=['POST'])
def add_work_form():
    """Handle form submissions."""
    try:
        # Extract form data
        uname_o = request.form.get('uname_o', '')
        uname_u = request.form.get('uname_u', '')
        tdiff_str = request.form.get('tdiff', '')
        secret = request.form.get('sev', '')
        
        # Validate required fields
        if not uname_o or not uname_u or not tdiff_str or not secret:
            return render_template_string(
                HTML_TEMPLATE, 
                error="All fields are required"
            )
        
        # Validate tdiff is an integer
        try:
            tdiff = int(tdiff_str)
            if tdiff <= 0:
                return render_template_string(
                    HTML_TEMPLATE,
                    error="Target difficulty must be a positive integer"
                )
        except ValueError:
            return render_template_string(
                HTML_TEMPLATE,
                error="Target difficulty must be an integer"
            )
        
        # Insert work item into database
        conn = get_db_connection(readonly=False)
        db_ws.insert_work(conn, uname_o, uname_u, tdiff)
        conn.close()
        
        return render_template_string(
            HTML_TEMPLATE,
            success="Work item added successfully"
        )
        
    except Exception as e:
        return render_template_string(
            HTML_TEMPLATE,
            error=f"An error occurred: {str(e)}"
        )


@app.route('/api/work-count', methods=['GET'])
def get_count():
    try:
        print(f"Received get-count")

        conn = get_db_connection(readonly=True)
        cursor = conn.cursor()
        cnt = db_ws.get_work_count(cursor)
        cursor.close()
        conn.close()

        return jsonify({"work_count": cnt}), 200

    except Exception as e:
        return jsonify({"error": str(e)}), 500


# URL parameter:
# - start_id: Start after this ID,exclusive
# - start_time: Start at this time, inclusive
# - limit: Optional, limit the number of entries returned (0=unlimited)
@app.route('/api/get-work-after-id', methods=['GET'])
def get_work_after_id():
    try:
        print(f"Received get-count")

        try:
            start_id = int(request.args.get('start_id'))
        except:
            raise Exception(f"Missing numeric 'start_id' parameter!")
        if start_id is None or start_id == 0:
            raise Exception(f"Invalid 'start_id' parameter '{start_id}'!")
        try:
            start_time = int(request.args.get('start_time'))
        except:
            raise Exception(f"Missing numeric 'start_time' parameter!")
        if start_time is None or start_time == 0:
            raise Exception(f"Invalid 'start_id' parameter '{start_time}'!")
        limit = 0
        try:
            limit = int(request.args.get('limit'))
        except:
            limit = 0

        conn = get_db_connection(readonly=True)
        cursor = conn.cursor()
        work_list = db_ws.get_work_after_id(cursor, start_id, start_time, limit)
        cursor.close()
        conn.close()

        asjson = list(map(lambda w: w.__dict__, work_list))
        return jsonify(asjson), 200

    except Exception as e:
        return jsonify({"error": str(e)}), 500


if __name__ == '__main__':
    app.run(debug=False, port=5004)
