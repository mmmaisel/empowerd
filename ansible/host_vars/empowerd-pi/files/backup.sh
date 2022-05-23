#!/bin/bash

BACKUP_PATH="/root/"

rm -r "$BACKUP_PATH/influx.bak"
influxd backup -portable "$BACKUP_PATH/influx.bak" > /dev/null
