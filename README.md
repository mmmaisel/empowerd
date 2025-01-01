# empowerd

Empowerd is a Linux daemon which empowers the offline smart home.

Currently, it supports monitoring of different power sources and
controlling appliances via modbus or Raspi GPIO pins.
GPIOs can be controlled manually via the Web-GUI.

The project focuses on code efficiency and robustness.

## Installation

The recommended way to install *empowerd* is to use the Debian package built
with "cargo deb". This will install all files (binary, main config,
logrotate config and systemd unit) to the correct locations and create a
separate user `empowerd` for the daemon.

In order to use *empowerd* a *PostgreSQL database* is required to store the
gathered data. For visualization you can use a compatible tool like
*Grafana*.

## Building

The build process uses cargo as top level build system to build the rust
components. To build the debian package, the "cargo deb" plugin is required.
It can be installed with `cargo install cargo-deb`.
Addtionally, "npm" is required to build the web GUI.

## Configuration

Set the correct database IP, port, username and password in the
*database* section of the *empowerd.conf*.

For every monitored datasource, add a *[[source]]* block to the config and
configure the required options. Common options for all sources are:

* *name*: The name of the source.
* *series_id*: The database ID of the series.
* *poll_interval*: The interval in second at which this source is monitored.
* *type*: The type of the datasource. This determines the protocol used for
  communication with the source and the asource specific options.

For the individual options of the different source types, the provided example
config. Usually, these options are IP addresses and ports, passwords or device
nodes.

## Postgres database setup (with Grafana)
Execute the following statements as superuser in the Postgres shell to
create a new database with two users. One for empowerd that manages the schema
and inserts data and another one with read-only access for Grafana.

```
CREATE DATABASE empowerd OWNER empowerd;
\c empowerd;
ALTER USER empowerd WITH PASSWORD 'password';
CREATE USER grafana WITH PASSWORD 'password';
ALTER DEFAULT PRIVILEGES FOR USER empowerd IN SCHEMA public GRANT SELECT ON TABLES TO grafana;
```

### Grafana Web-UI

To use the Web-UI, you need a web-server which run Grafana and forwards the
`/graphql` URL.

An example nginx configuration can be found at
"data/nginx-site.conf" or "usr/share/doc/empowerd/" after installing the
Debian package.

The Grafana UI is installed by the `empowerd-grafana` package to the
`/usr/share/empowerd/grafana` directory and needs to be symlinked into
your Grafana plugins directory. Then, the unsigned plugin has to be
whitelisted in the Grafana config.

Finally, the plugin has to be configured in the Grafana plugin admin section.
There you have to set the PostgreSQL datasource and the series IDs from
the backend config.

To use the controls section of the GUI you have to configure the listen address,
a username and an argon2 password hash in the *[graphql]* section of the
config file. Currently, the controlled GPIOs are configured in this section
as well.

## License

*empowerd* is licensed under the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

### Icon License
Grafana plugin logo is based on `battery-charging` icon from
[Oxygen icon theme](https://github.com/KDE/oxygen-icons), licensed under the
GNU LGPLv3 or later.

`config.svg` icon in `grafana-gui/src/img` directory is based on
`configure` icon from [Oxygen icon theme](https://github.com/KDE/oxygen-icons),
licensed under the GNU LGPLv3 or later.
