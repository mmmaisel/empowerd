# Empowerd-GUI

Empowerd Grafana GUI

## Overview / Introduction

UI for the *empowerd* service that displays live statistics of the monitored
power sources and sinks as well as controls outputs.

## Requirements
*empowerd* backend service and web server that proxies empowerd API requests.
Grafana, empowerd and the API reverse proxy must be run on the same server.

## Getting Started
* Setup PostgreSQL database for empowerd and add a read-only Grafana user.
* Setup empowerd, see main documentation linked below.
* Select the datasource and link the series IDs in the config JSON strings.

## Documentation
See main `README.md` file in the
[empowerd repository](https://github.com/mmmaisel/empowerd).
