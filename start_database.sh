#!/bin/bash

docker run -d \
    -e POSTGRES_PASSWORD=pass \
    -e POSTGRES_DB=db \
    -e POSTGRES_USER=user \
    -p 5432:5432 \
    postgres

sqlx migrate run