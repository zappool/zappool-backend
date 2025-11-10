#!/bin/bash
#
# Script for database backup
# Operations:
# 1 - copy the db dir to backup/current
# 2 - copy backup/current to backup/hourly
# 3 - copy backup/current to backup/dailly
# 4 - copy backup/current to backup/weekly
#
# Can be set in crontab like this:
#
#  0 * * * * cd FOLDER/zappool && ./scripts/backup_data.sh 1
# 10 * * * * cd FOLDER/zappool && ./scripts/backup_data.sh 2
# 20 0 * * * cd FOLDER/zappool && ./scripts/backup_data.sh 3
# 30 0 * * 0 cd FOLDER/zappool && ./scripts/backup_data.sh 4
#
# crontab -u USER -l
# crontab -u USER -e
#

# Obtain DB_DIR from .env (should be without spaces and quotes)
export `grep DB_DIR .env`
echo "DB dir: "$DB_DIR
if [ -z "$DB_DIR"]; then
    echo "Missing DB_DIR from .env"
    exit -3
fi

BACKUP_ROOT="./backup"

do_copy()
{
    SRC=$1
    DEST=$2
    echo "===Copying, src: "$SRC"  dest: "$DEST
    ls -dl $SRC
    # ls $DEST
    rsync -av --mkpath $SRC $DEST
    echo "===Done, dest:"
    ls -alt $DEST
    echo "Backup:"
    ls -alt $BACKUP_ROOT
    du -sk $DEST
    du -sk $BACKUP_ROOT/*
    cat $BACKUP_ROOT"/current/timestamp"
}

# echo $1
if [ -z "$1" ]; then
    echo "Usage:  one argument:"
    echo "  0 - copy the db dir to backup/current"
    echo "  1 - copy backup/current to backup/hourly"
    echo "  2 - copy backup/current to backup/dailly"
    echo "  3 - copy backup/current to backup/weekly"
    exit -1
else
    case "$1" in
        1)
            do_copy $DB_DIR $BACKUP_ROOT"/current"
            date -u -Iseconds > $BACKUP_ROOT"/current/timestamp"
            ;;
        2)
            do_copy $BACKUP_ROOT"/current/*" $BACKUP_ROOT"/hourly"
            ;;
        3)
            do_copy $BACKUP_ROOT"/current/*" $BACKUP_ROOT"/daily"
            ;;
        4)
            do_copy $BACKUP_ROOT"/current/*" $BACKUP_ROOT"/weekly"
            ;;
        *)
            echo "Unknown argument "$1
            exit -2
            ;;
    esac
fi

