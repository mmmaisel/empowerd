# empowerd

Empowerd is a Linux daemon which empowers the offline smart home.

## Installation

The recommended way to install *empowerd* is to use the Debian package built
with "cargo deb". This will install all files (binary, main config,
logrotate config and systemd unit) to the correct locations and create a
separate user `empowerd` for the daemon.

In order to use *empowerd* an *Influx database* is required to store the
gathered data. For visualization you can use an Influx compatible tool like
*Grafana*.

## Configuration

Set the correct Influx database IP, port, username and password in the
*database* section of the *empowerd.conf*.

For every monitored datasource, add a *[[source]]* block to the config and
configure the required options. Common options for all sources are:

* *name*: The name of the used Influx database measurement.
* *poll_interval*: The interval in second at which this source is monitored.
* *type*: The type of the datasource. This determines the protocol used for
  communication with the source and the asource specific options.

For the individual options of the different source types, the provided example
config. Usually, these options are IP addresses and ports, passwords or device
nodes.

## License

*empowerd* is licensed under the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version.
