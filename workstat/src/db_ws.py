from dto_ws import Work

# from datetime import datetime, UTC
import sqlite3


def db_setup_1(conn: sqlite3.Connection):
    cursor = conn.cursor()

    # Create table ORUser  (ORiginal User)
    cursor.execute("""
                   CREATE TABLE ORUSER
                   (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameO VARCHAR(100), UNameO_wrkr VARCHAR(100), UNameU_wrkr VARCHAR(100), TimeAdd INTEGER)
                   """)

    # Create table USUser  (UpStream User)
    cursor.execute("""
                   CREATE TABLE USUSER
                   (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameU VARCHAR(100), TimeAdd INTEGER)
                   """)

    # Create table WORK
    # UNameO -- original username (without worker name)
    # UNameO_wrkr - Optional worker in the original username
    # UNameU -- upstream username (without worker name)
    # UNameU_wrkr - Worker in the upstream username
    # TDiff - The target diffuclty of the work request
    # TimeAdd - Time when package was added
    # TimeCalc - Time when payout was calculated
    # CalcPayout - The calculated estimated payout, sats
    cursor.execute("""
        CREATE TABLE WORK (
                   Id INTEGER PRIMARY KEY AUTOINCREMENT,
                   UNameO INTEGER,
                   UNameU INTEGER,
                   TDiff INTEGER,
                   TimeAdd INTEGER,
                   TimeCalc INTEGER,
                   CalcPayout INTEGER,
                   FOREIGN KEY (UNameO) REFERENCES ORUSER(Id)
                   FOREIGN KEY (UNameU) REFERENCES USUSER(Id)
        )
        """)
    cursor.execute("CREATE INDEX WorkTimeAdd ON WORK (TimeAdd)")

    # Commit changes and close connection
    conn.commit()


# Get or insert orignal username
# Note: it does'n commit
def get_or_insert_orig_user(conn, uname_o: str, uname_o_wrkr: str, uname_u_wrkr: str, time_add: int) -> int:
    cursor = conn.cursor()

    cursor.execute("SELECT Id FROM ORUSER WHERE UNameO = ? AND UNameO_wrkr = ?", (uname_o, uname_o_wrkr))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            # Found in DB
            cursor.close()
            return rows[0][0]

    # Not found, insert
    cursor.execute(
        """
            INSERT INTO ORUSER 
                (UNameO, UNameO_wrkr, UNameU_wrkr, TimeAdd)
            VALUES
                (?, ?, ?, ?)
        """,
        (uname_o, uname_o_wrkr, uname_u_wrkr, time_add)
    )

    # Query again
    cursor.execute("SELECT Id FROM ORUSER WHERE UNameO = ? AND UNameO_wrkr = ?", (uname_o, uname_o_wrkr))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            # Found in DB
            cursor.close()
            return rows[0][0]

    cursor.close()
    raise Exception(f"Could not insert original user {uname_o} {uname_o_wrkr}")


# Get or insert upstream username
# Note: it does'n commit
def get_or_insert_us_user(conn, uname_u: str, time_add: int) -> int:
    cursor = conn.cursor()

    cursor.execute("SELECT Id FROM USUSER WHERE UNameU = ?", (uname_u,))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            # Found in DB
            cursor.close()
            return rows[0][0]

    # Not found, insert
    cursor.execute(
        """
            INSERT INTO USUSER 
                (UNameU, TimeAdd)
            VALUES
                (?, ?)
        """,
        (uname_u, time_add)
    )

    # Query again
    cursor.execute("SELECT Id FROM USUSER WHERE UNameU = ?", (uname_u,))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            # Found in DB
            cursor.close()
            return rows[0][0]

    cursor.close()
    raise Exception(f"Could not insert upstream user {uname_u}")


# May throw if cannot connect
def insert_work_struct(conn, w: Work):
    cursor = conn.cursor()

    user_orig_id = get_or_insert_orig_user(conn, w.uname_o, w.uname_o_wrkr, w.uname_u_wrkr, w.time_add)
    # print(f"user_orig_id = {user_orig_id}")
    user_us_id = get_or_insert_us_user(conn, w.uname_u, w.time_add)
    # print(f"user_us_id = {user_us_id}")

    cursor.execute(
        """
            INSERT INTO Work 
                (UNameO, UNameU, TDiff, TimeAdd, TimeCalc, CalcPayout)
            VALUES
                (?, ?, ?, ?, ?, ?)
        """,
        (user_orig_id, user_us_id, w.tdiff, w.time_add, w.time_calc, w.calc_payout)
    )

    cursor.close()
    conn.commit()


# May throw if cannot connect
def insert_work(conn, uname_o: str, uname_u: str, tdiff: int):
    work = Work.new(uname_o, uname_u, tdiff)
    return insert_work_struct(conn, work)


def get_all_work_limit(cursor) -> list[Work]:
    cursor.execute("""
        SELECT WORK.Id, ORUSER.UNameO, ORUSER.UNameO_wrkr, USUSER.UNameU, ORUSER.UNameU_wrkr, WORK.TDiff, WORK.TimeAdd, WORK.TimeCalc, WORK.CalcPayout
        FROM WORK
        LEFT OUTER JOIN ORUSER
        ON WORK.UNameO = ORUSER.Id
        LEFT OUTER JOIN USUSER
        ON WORK.UNameU = USUSER.Id
        ORDER BY WORK.TimeAdd DESC
        LIMIT 1000
    """)
    rows = cursor.fetchall()
    res = []
    for row in rows:
        if len(row) >= 9:
            work = Work(row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8])
            res.append(work)
    return res


def get_work_after_id(cursor, start_id: int, start_time: int) -> list[Work]:
    cursor.execute("""
        SELECT WORK.Id, ORUSER.UNameO, ORUSER.UNameO_wrkr, USUSER.UNameU, ORUSER.UNameU_wrkr, WORK.TDiff, WORK.TimeAdd, WORK.TimeCalc, WORK.CalcPayout
        FROM WORK
        LEFT OUTER JOIN ORUSER
        ON WORK.UNameO = ORUSER.Id
        LEFT OUTER JOIN USUSER
        ON WORK.UNameU = USUSER.Id
        WHERE WORK.Id > ? AND WORK.TimeAdd >= ?
        ORDER BY WORK.Id ASC
    """, (start_id, start_time,))
    rows = cursor.fetchall()
    res = []
    for row in rows:
        if len(row) >= 9:
            work = Work(row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8])
            res.append(work)
    return res


def get_work_count(cursor) -> int:
    cursor.execute("SELECT COUNT(*) FROM WORK")
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            cnt = rows[0][0]
    # print(cnt)
    return cnt

