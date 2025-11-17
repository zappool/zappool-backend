import db_ws
from main import app, init_app

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

from flask import Flask, request, jsonify, render_template_string
import dotenv
import requests
import os
import sqlite3
import _thread
import time
import unittest


def run_api_server(app):
    init_app()
    app.run(debug=False, port=8000)

class WorkstatApiTestClass(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        print("setUpClass")

        cls.url_root = "http://localhost:8000/api"
        cls.dbfile = "/tmp/workstat.db"

        os.environ["DB_DIR"] = "/tmp"
        os.environ["WORKSTAT_SECRET"] = "secret"

        dotenv.load_dotenv()
        # print(f"DB_DIR: {os.getenv("DB_DIR")}")
        # cls.assertEqual(os.getenv("DB_DIR"), "/tmp")

        cls.recreate_db(cls)

        _th = _thread.start_new(run_api_server, (app,))
        time.sleep(0.05)

    # def setUp(self):
    #     print("setUp")
        # self.url_root = "http://localhost:8000/api"
        # self.dbfile = "/tmp/workstat.db"

        # dotenv.load_dotenv()
        # # print(f"DB_DIR: {os.getenv("DB_DIR")}")
        # self.assertEqual(os.getenv("DB_DIR"), "/tmp")

        # self.recreate_db()

    def recreate_db(self):
        print(f"Temp DB used: {self.dbfile}")
        if os.path.isfile(self.dbfile):
            os.remove(self.dbfile)
        conn = sqlite3.connect(self.dbfile)
        db_ws.db_setup_1(conn)
        conn.close()

    def get_count_check(self, expected_count: int):
        url = f"{self.url_root}/work-count"
        response = requests.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.text, "{\"work_count\":" + str(expected_count) + "}\n")

    def test_empty_count(self):
        self.recreate_db()
        self.get_count_check(0)

    def insert_work_item(self, user: str):
        work1 = '{"uname_o": "' + user + '", "uname_u": "upstream1", "tdiff": 100, "sec": "secret"}'
        url = f"{self.url_root}/work-insert"
        response = requests.post(url, data=work1, headers={"Content-Type": "application/json"})
        self.assertEqual(response.status_code, 201)
        # print(response.text)
        self.assertEqual(response.text, "{\"message\":\"Work item added successfully\"}\n")

    def test_insert_and_count(self):
        self.recreate_db()
        self.get_count_check(0)
        self.insert_work_item("user1")
        self.get_count_check(1)
        self.insert_work_item("user2")
        self.get_count_check(2)

    def test_insert_and_get(self):
        self.recreate_db()
        for i in range(10):
            self.insert_work_item(f"user{i}")
        self.get_count_check(10)

        url = f"{self.url_root}/get-work-after-id?start_id=5&start_time=1"
        response = requests.get(url)
        self.assertEqual(response.status_code, 200)
        result = response.json()
        self.assertEqual(len(result), 5)
        self.assertEqual(result[0]['db_id'], 6)
        self.assertEqual(result[0]['uname_o'], 'user5')

if __name__ == "__main__":
    unittest.main() # run all tests


