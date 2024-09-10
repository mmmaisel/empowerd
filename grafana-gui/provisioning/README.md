This directory contains the configuration files used for a development setup.
The files have to go to the following locations:

* grafana.ini
  Copy this file to `/etc/grafana/grafana.ini`
* inject-livereload.sh
  Execute this script on the development server to enable live-reload from
  npm development server.

Bind mounts:
* This directory:
  `/home/user/empowerd/grafana-gui/provisioning /etc/grafana/provisioning none defaults,bind 0 0`
* The `dist` directory one level above:
  `/home/user/empowerd/grafana-gui/dist /var/lib/grafana/plugins/empowerd none defaults,bind 0 0`
