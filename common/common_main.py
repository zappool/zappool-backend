import dotenv
import os
import sys


# Return the data dir: the first arg or "."
def get_data_dir() -> str:
    dotenv.load_dotenv()

    data_dir_from_env = os.getenv("DB_DIR")
    if data_dir_from_env == None:
        local_dir = os.getcwd()
        print(f"Using local directory as data dir, ({local_dir})")
        data_dir = local_dir
    else:
        data_dir = data_dir_from_env
        print(f"Using data dir from env: '{data_dir}'")
    return data_dir


# Check and return full path of a DB file
def get_db_file(db_file_name: str, create_mode: bool = False) -> str:
    data_dir = get_data_dir()
    if create_mode:
        db_file_name = "_new_" + db_file_name
    dbfile = data_dir + "/" + db_file_name
    if not create_mode:
        if not os.path.exists(dbfile):
            print(f"DB file does not exist! {dbfile}")
            sys.exit(-1)
    print(f"Using data file: '{dbfile}'")
    return dbfile


def get_db_update_versions_from_args(default_to: int) -> tuple[int, int]:
    vto = default_to
    vfrom = vto - 1

    if len(sys.argv) >= 3:
        vfrom = int(sys.argv[1])
        vto = int(sys.argv[2])

    print(f"DB update versions: v{vfrom} --> v{vto}")
    return [vto, vfrom]
