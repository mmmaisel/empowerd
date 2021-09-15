# empowerd

Empowerd is a Linux daemon which empowers the offline smart home.

Currently, it supports monitoring of different power sources as well as
controlling things like valves on Raspi GPIO pins via a web GUI.

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

### Icon License
Icons `config.svg`, `logout.svg` and `status.svg` in `gui/public` directory were
taken from the [`Humanity` icon theme](https://launchpad.net/humanity), licensed
under the GNU GPLv2. Original `AUTHORS` file follows below:

```
####################
ABOUT:             #
####################

Humanity is designed and developed by Daniel Foré <Daniel.p.Fore@gmail.com>, Jonian Guveli <jonian.guveli@gmail.com>, and K.Vishnoo Charan Reddy<foo.mac.v@gmail.com>.

GNOME icons and Humanity icons are all licensed under the GPL.

This package is licensed under GNU General Public License version 2.

Icons based or directly from GNOME and other GNOME projects, licensed GPL.
	You can visit the GNOME website here:
		http://www.gnome.org/

Icons based on Tango sources or taken from the Tango project are public domain.
	You can visit the Tango project website here:
		http://tango.freedesktop.org/Tango_Desktop_Project
```
